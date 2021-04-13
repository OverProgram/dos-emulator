CPU 286

    mov ax, 0x10
    mov dx, 0x20
    xchg ax, dx
    nop
    mov bx, 5
    mov ax, 5
    xlat
    nop
