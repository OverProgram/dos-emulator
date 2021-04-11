CPU 286

    mov si, 0
    mov di, 0
    mov ax, 10
    cld
lop_cmps:
    cmpsb
    je lop_cmps
    mov ax, 20
    nop
