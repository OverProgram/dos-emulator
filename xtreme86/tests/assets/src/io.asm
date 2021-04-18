CPU 286

    mov ax, 0x1234
    out 0xF2, ax
    in ax, 0xF2
    nop

    mov dx, 0xEEDA
    mov al, 0x56
    out dx, al
    in al, dx
    nop
    int 0x12
    mov al, 0x78
    out dx, al
    in ax, dx
    nop
