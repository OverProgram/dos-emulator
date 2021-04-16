CPU 286

    mov bx, 0x00
    mov byte [es:bx], 0x05
    nop

    mov bp, 0x06
    mov word [bp], 0xFFFF
    nop
