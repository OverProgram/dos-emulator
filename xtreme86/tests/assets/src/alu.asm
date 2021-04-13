CPU 286

    mov ah, 0
    nop
    mov al, 9
    nop
    mov bl, 9
    nop
    add al, bl
    aaa
    nop

    mov ax, 5
    stc
    adc ax, 5
    nop

    stc
    sbb ax, 10
    nop

    mov ax, 0xFFFF
    xor al, al
    nop

    not ax
    nop

    neg al
    nop

    mov ax, 0x0A0E
    aad
    nop

    mov ax, 0x0105
    sub al, 0x0A
    aas
    nop

    mov al, 0x79
    mov bl, 0x35
    add al, bl
    daa
    nop
