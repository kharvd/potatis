	.org $a580

reset:
loop:
	nop
	jmp loop

	.org $fffc
	.word reset
	.word $0000