CPU 286

    mov ax, 6
    int 0x00
    nop
    mov ax, 10
    bound ax, [0]
    nop
    mov ax, 0xFFFF
    inc ax
    into
    nop

routine:
    mov ax, 5
    iret
