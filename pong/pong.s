WIDTH = 240
HEIGHT = 192

SCREEN_X = $8000
SCREEN_Y = $8001
SCREEN_COLOR = $8002
SCREEN_COMMAND = $8003
SCREEN_COMMAND_NOP = $00
SCREEN_COMMAND_DRAW = $01
SCREEN_COMMAND_CLEAR = $02
SCREEN_COMMAND_FLUSH = $03

DrawX = $0
DrawY = $1
RectX = $2
RectY = $3
RectW = $4
RectH = $5
VelocityX = $6
VelocityY = $7

	.org $a580

reset:
    lda #1
    sta RectX
    lda #1
    sta RectY
    lda #10
    sta RectW
    lda #10
    sta RectH
    lda #16
    sta VelocityX
    lda #5
    sta VelocityY

loop:
    lda #SCREEN_COMMAND_CLEAR
    sta SCREEN_COMMAND

    jsr draw_rect

    lda #SCREEN_COMMAND_FLUSH
    sta SCREEN_COMMAND

    jsr advance_rect

	jmp loop

; ----------------------------------------
putpixel:
    pha
    lda DrawX
    sta SCREEN_X
    lda DrawY
    sta SCREEN_Y
    lda #$ff
    sta SCREEN_COLOR
    lda #SCREEN_COMMAND_DRAW
    sta SCREEN_COMMAND

    pla
    rts
; ----------------------------------------

draw_rect:
    pha

    lda RectX
    sta DrawX
loop_x:
    sec
    sbc RectX
    cmp RectW
    beq end_loop_x

    lda RectY
    sta DrawY
loop_y:
    sec
    sbc RectY
    cmp RectH
    beq end_loop_y

    jsr putpixel
    inc DrawY
    lda DrawY
    jmp loop_y
end_loop_y:
    
    inc DrawX
    lda DrawX
    jmp loop_x
end_loop_x:

    pla
    rts
; ----------------------------------------

advance_rect:
    pha

    lda RectX
    clc
    adc VelocityX
    sta RectX
    adc RectW
    cmp #WIDTH
    bcc check_x_pos

fix_x_pos:
    sec
    lda #0
    sbc VelocityX
    sta VelocityX
    adc RectX
    sta RectX
    jmp check_y

check_x_pos:
    lda #WIDTH
    cmp RectX
    bcc fix_x_pos

check_y:
    lda RectY
    clc
    adc VelocityY
    sta RectY
    adc RectH
    cmp #HEIGHT
    bcc check_y_pos

fix_y_pos:
    sec
    lda #0
    sbc VelocityY
    sta VelocityY
    adc RectY
    sta RectY
    jmp check_end

check_y_pos:
    lda #HEIGHT
    cmp RectY
    bcc fix_y_pos

check_end:

    pla
    rts

	.org $fffc
	.word reset
	.word $0000