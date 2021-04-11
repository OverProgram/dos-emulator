CPU 286

    mov bx, 5
    lea ax, [bx+4]
    nop
    mov al, 0xFF
    cbw
    nop
    mov dx, 0
    cwd
    nop
