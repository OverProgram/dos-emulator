CPU 286

    mov ax, 0x4000
    mov es, ax

    mov bx, 0
load_loop:
    mov ch, [es:bx]
    mov [ds:bx], ch
    inc bx
    cmp ch, 0
    jne load_loop

    mov dx, 0
    int 0x21
    nop
