CPU 286

    mov ax, 9
    mov dx, 9
    cmp ax, dx
    je exit
    nop

exit:
    mov ax, 0x30
    nop
