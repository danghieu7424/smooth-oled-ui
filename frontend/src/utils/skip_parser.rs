// frontend/src/utils/skip_parser.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TimeSpec {
    Absolute(f64),
    FromEnd(f64),
    End,
    Ignore,
}

#[derive(Debug, Clone)]
pub struct SkipRule {
    pub start: TimeSpec,
    pub end: TimeSpec,
    pub includes: Vec<String>,
    pub excludes: Vec<String>,
    pub message: Option<String>,
}

impl SkipRule {
    pub fn is_match(&self, filename: &str) -> bool {
        if self.excludes.iter().any(|e| wildcard_match(e, filename)) {
            return false;
        }
        if self.includes.is_empty() {
            return true;
        }
        self.includes.iter().any(|i| wildcard_match(i, filename))
    }

    pub fn resolve_start(&self, duration: f64) -> f64 {
        match self.start {
            TimeSpec::Absolute(t) => t,
            TimeSpec::FromEnd(t) => duration - t,
            TimeSpec::End => duration,
            TimeSpec::Ignore => 0.0,
        }
    }

    pub fn resolve_end(&self, duration: f64) -> f64 {
        match self.end {
            TimeSpec::Absolute(t) => t,
            TimeSpec::FromEnd(t) => duration - t,
            TimeSpec::End => duration,
            TimeSpec::Ignore => 0.0,
        }
    }
}

pub fn parse_skip_rules(content: &str) -> Vec<SkipRule> {
    let mut rules = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        if let Some(rule) = parse_line(line) {
            rules.push(rule);
        }
    }
    rules
}

fn parse_line(line: &str) -> Option<SkipRule> {
    let start_bracket = line.find('[')?;
    let end_bracket = line.find(']')?;
    
    if start_bracket >= end_bracket { return None; }
    
    let time_range = &line[start_bracket + 1..end_bracket];
    
    let (start_spec, end_spec) = if time_range.eq_ignore_ascii_case("IGNORE") {
        (TimeSpec::Ignore, TimeSpec::Ignore)
    } else {
        let parts: Vec<&str> = time_range.split('-').collect();
        if parts.len() != 2 { return None; }
        
        (parse_time_spec(parts[0].trim())?, parse_time_spec(parts[1].trim())?)
    };
    
    let rest = line[end_bracket + 1..].trim();
    
    let mut includes = Vec::new();
    let mut excludes = Vec::new();
    let mut message = None;
    
    if let Some(b1) = rest.find('{') {
        if let Some(b2) = rest[b1 + 1..].find('}') {
            message = Some(rest[b1 + 1 .. b1 + 1 + b2].to_string());
        }
    }
    
    let mut current_search = rest;
    while let Some(q1) = current_search.find('"') {
        if let Some(q2) = current_search[q1 + 1..].find('"') {
            let file_str = current_search[q1 + 1 .. q1 + 1 + q2].to_string();
            let is_exclude_prefix = q1 > 0 && current_search.chars().nth(q1 - 1) == Some('!');
            
            if file_str.starts_with('!') {
                for exp in expand_pattern(&file_str[1..]) {
                    excludes.push(exp);
                }
            } else if is_exclude_prefix {
                for exp in expand_pattern(&file_str) {
                    excludes.push(exp);
                }
            } else {
                for exp in expand_pattern(&file_str) {
                    includes.push(exp);
                }
            }
            current_search = &current_search[q1 + 1 + q2 + 1..];
        } else {
            break;
        }
    }
    
    Some(SkipRule {
        start: start_spec,
        end: end_spec,
        includes,
        excludes,
        message,
    })
}

fn parse_time_spec(s: &str) -> Option<TimeSpec> {
    if s == "(END)" || s.eq_ignore_ascii_case("END") {
        return Some(TimeSpec::End);
    }
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        if let Some(t) = parse_time(inner) {
            return Some(TimeSpec::FromEnd(t));
        }
    }
    if let Some(t) = parse_time(s) {
        return Some(TimeSpec::Absolute(t));
    }
    None
}

fn parse_time(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split(':').collect();
    let seconds;
    
    if parts.len() == 3 {
        let h = parts[0].parse::<f64>().ok()?;
        let m = parts[1].parse::<f64>().ok()?;
        let s = parts[2].parse::<f64>().ok()?;
        seconds = h * 3600.0 + m * 60.0 + s;
    } else if parts.len() == 2 {
        let m = parts[0].parse::<f64>().ok()?;
        let s = parts[1].parse::<f64>().ok()?;
        seconds = m * 60.0 + s;
    } else if parts.len() == 1 {
        seconds = parts[0].parse::<f64>().ok()?;
    } else {
        return None;
    }
    Some(seconds)
}

fn expand_pattern(pattern: &str) -> Vec<String> {
    if let Some(start_brace) = pattern.find('{') {
        if let Some(dot_dot) = pattern[start_brace..].find("..") {
            let dot_dot_idx = start_brace + dot_dot;
            if let Some(end_brace) = pattern[dot_dot_idx..].find('}') {
                let end_brace_idx = dot_dot_idx + end_brace;
                
                let start_num_str = &pattern[start_brace + 1..dot_dot_idx];
                let end_num_str = &pattern[dot_dot_idx + 2..end_brace_idx];
                
                if let (Ok(start), Ok(end)) = (start_num_str.parse::<u32>(), end_num_str.parse::<u32>()) {
                    let padding = start_num_str.len();
                    let mut expanded = Vec::new();
                    let prefix = &pattern[..start_brace];
                    let suffix = &pattern[end_brace_idx + 1..];
                    
                    for i in start..=end {
                        let num_str = format!("{:0>width$}", i, width=padding);
                        expanded.push(format!("{}{}{}", prefix, num_str, suffix));
                    }
                    return expanded;
                }
            }
        }
    }
    vec![pattern.to_string()]
}

fn wildcard_match(pattern: &str, target: &str) -> bool {
    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;
    
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = target.chars().collect();
    
    while t_idx < t.len() {
        if p_idx < p.len() && (p[p_idx] == '?' || p[p_idx] == t[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p.len() && p[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = t_idx;
            p_idx += 1;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }
    
    while p_idx < p.len() && p[p_idx] == '*' {
        p_idx += 1;
    }
    
    p_idx == p.len()
}
