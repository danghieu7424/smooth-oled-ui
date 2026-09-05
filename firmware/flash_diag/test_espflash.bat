echo DEADBEEF > test.bin
espflash write-bin 0x340000 test.bin
espflash read-flash 0x340000 16 readback.bin
certutil -encodehex readback.bin readback.hex
type readback.hex
