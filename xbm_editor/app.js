const canvas = document.getElementById('pixel-canvas');
const ctx = canvas.getContext('2d');
const inputWidth = document.getElementById('grid-width');
const inputHeight = document.getElementById('grid-height');
const codeOutput = document.getElementById('code-output');
const btnUpload = document.getElementById('btn-upload');
const fileInput = document.getElementById('file-input');
const btnClear = document.getElementById('btn-clear');
const btnLoadCode = document.getElementById('btn-load-code');
const btnCopy = document.getElementById('btn-copy');

let gridW = 24;
let gridH = 24;
let pixelSize = 16;
let pixels = [];
let isDrawing = false;
let drawMode = true; // true = draw, false = erase

function init() {
    gridW = parseInt(inputWidth.value) || 24;
    gridH = parseInt(inputHeight.value) || 24;
    
    // Resize grid visually
    const containerW = document.getElementById('canvas-container').clientWidth - 48;
    const containerH = document.getElementById('canvas-container').clientHeight - 48;
    
    pixelSize = Math.min(Math.floor(containerW / gridW), Math.floor(containerH / gridH), 24);
    if (pixelSize < 4) pixelSize = 4;

    canvas.width = gridW * pixelSize;
    canvas.height = gridH * pixelSize;
    
    // Clear pixel array
    pixels = new Array(gridW * gridH).fill(false);
    
    drawGrid();
    generateCode();
}

function drawGrid() {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    // Draw pixels
    ctx.fillStyle = '#00e5ff';
    for (let y = 0; y < gridH; y++) {
        for (let x = 0; x < gridW; x++) {
            if (pixels[y * gridW + x]) {
                ctx.fillRect(x * pixelSize, y * pixelSize, pixelSize, pixelSize);
            }
        }
    }
    
    // Draw lines
    ctx.strokeStyle = '#1f2937';
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (let x = 0; x <= gridW; x++) {
        ctx.moveTo(x * pixelSize, 0);
        ctx.lineTo(x * pixelSize, canvas.height);
    }
    for (let y = 0; y <= gridH; y++) {
        ctx.moveTo(0, y * pixelSize);
        ctx.lineTo(canvas.width, y * pixelSize);
    }
    ctx.stroke();
}

function getCoords(e) {
    const rect = canvas.getBoundingClientRect();
    const x = Math.floor((e.clientX - rect.left) / pixelSize);
    const y = Math.floor((e.clientY - rect.top) / pixelSize);
    return {x, y};
}

canvas.addEventListener('mousedown', (e) => {
    isDrawing = true;
    const {x, y} = getCoords(e);
    if (x >= 0 && x < gridW && y >= 0 && y < gridH) {
        drawMode = !pixels[y * gridW + x];
        pixels[y * gridW + x] = drawMode;
        drawGrid();
        generateCode();
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (!isDrawing) return;
    const {x, y} = getCoords(e);
    if (x >= 0 && x < gridW && y >= 0 && y < gridH) {
        if (pixels[y * gridW + x] !== drawMode) {
            pixels[y * gridW + x] = drawMode;
            drawGrid();
            generateCode();
        }
    }
});

window.addEventListener('mouseup', () => isDrawing = false);

// Convert pixels to XBM format (LSB first)
function generateCode() {
    const bytesPerRow = Math.ceil(gridW / 8);
    const totalBytes = bytesPerRow * gridH;
    let bytes = new Uint8Array(totalBytes);
    
    for (let y = 0; y < gridH; y++) {
        for (let x = 0; x < gridW; x++) {
            if (pixels[y * gridW + x]) {
                const byteIdx = y * bytesPerRow + Math.floor(x / 8);
                const bitIdx = x % 8;
                bytes[byteIdx] |= (1 << bitIdx);
            }
        }
    }
    
    let code = `static const unsigned char icon_custom[] U8X8_PROGMEM = {\n  `;
    for (let i = 0; i < bytes.length; i++) {
        let hex = bytes[i].toString(16).padStart(2, '0');
        code += `0x${hex}`;
        if (i < bytes.length - 1) {
            code += `, `;
            if ((i + 1) % 12 === 0) code += `\n  `;
        }
    }
    code += `\n};`;
    codeOutput.value = code;
}

// Convert XBM code back to pixels
function loadCode() {
    const code = codeOutput.value;
    const matches = code.match(/0x[0-9a-fA-F]{2}/g);
    if (!matches) {
        alert("No valid XBM hex data found in the text area.");
        return;
    }
    
    const bytesPerRow = Math.ceil(gridW / 8);
    if (matches.length > bytesPerRow * gridH) {
        alert(`Warning: The loaded array has ${matches.length} bytes, but current canvas size expects ${bytesPerRow * gridH} bytes. Image may be truncated or overflow.`);
    }
    
    pixels.fill(false);
    
    for (let i = 0; i < Math.min(matches.length, bytesPerRow * gridH); i++) {
        let byte = parseInt(matches[i], 16);
        let y = Math.floor(i / bytesPerRow);
        let xOffset = (i % bytesPerRow) * 8;
        
        for (let bit = 0; bit < 8; bit++) {
            let x = xOffset + bit;
            if (x < gridW) {
                pixels[y * gridW + x] = (byte & (1 << bit)) !== 0;
            }
        }
    }
    drawGrid();
}

// Handle Image Upload
btnUpload.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', (e) => {
    const file = e.target.files[0];
    if (!file) return;
    
    const reader = new FileReader();
    reader.onload = (event) => {
        const img = new Image();
        img.onload = () => {
            const tempCanvas = document.createElement('canvas');
            tempCanvas.width = gridW;
            tempCanvas.height = gridH;
            const tCtx = tempCanvas.getContext('2d');
            
            // Fill white background first (so transparent becomes white, i.e., OFF)
            tCtx.fillStyle = 'white';
            tCtx.fillRect(0, 0, gridW, gridH);
            
            // Draw image centered and scaled
            const scale = Math.min(gridW / img.width, gridH / img.height);
            const w = img.width * scale;
            const h = img.height * scale;
            const x = (gridW - w) / 2;
            const y = (gridH - h) / 2;
            
            tCtx.drawImage(img, x, y, w, h);
            const imgData = tCtx.getImageData(0, 0, gridW, gridH).data;
            
            for (let i = 0; i < imgData.length; i += 4) {
                const r = imgData[i];
                const g = imgData[i+1];
                const b = imgData[i+2];
                // Convert to grayscale, threshold at 128. Darker pixels become ON (white on OLED).
                const gray = (r * 0.299 + g * 0.587 + b * 0.114);
                const pIdx = i / 4;
                pixels[pIdx] = gray < 128;
            }
            drawGrid();
            generateCode();
        };
        img.src = event.target.result;
    };
    reader.readAsDataURL(file);
});

btnClear.addEventListener('click', () => {
    pixels.fill(false);
    drawGrid();
    generateCode();
});

btnLoadCode.addEventListener('click', loadCode);

btnCopy.addEventListener('click', () => {
    codeOutput.select();
    document.execCommand('copy');
    const oldText = btnCopy.innerText;
    btnCopy.innerText = "Copied!";
    setTimeout(() => btnCopy.innerText = oldText, 2000);
});

inputWidth.addEventListener('change', init);
inputHeight.addEventListener('change', init);
window.addEventListener('resize', () => {
    // Re-calculate pixel size without clearing pixels
    const containerW = document.getElementById('canvas-container').clientWidth - 48;
    const containerH = document.getElementById('canvas-container').clientHeight - 48;
    pixelSize = Math.min(Math.floor(containerW / gridW), Math.floor(containerH / gridH), 24);
    if (pixelSize < 4) pixelSize = 4;
    canvas.width = gridW * pixelSize;
    canvas.height = gridH * pixelSize;
    drawGrid();
});

// Initialize
init();
