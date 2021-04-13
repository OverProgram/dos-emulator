CPU 286

ORG 0x103F0

    mov ax, 9
    mov dx, 9
    cmp ax, dx
    je exit
    nop
cont:
    mov ax, 0x10
    mov dx, 0x10
    test dx, 0x80
    je exit
    nop

exit:
    mov ax, 0x30
    nop
    jmp cont
