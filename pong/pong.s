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

param1 = $f0
param2 = $f1
param3 = $f2
param4 = $f3

	.org $a580

; ----------------------------------------
reset:
    lda #1
    sta RectX
    lda #1
    sta RectY
    lda #5
    sta RectW
    lda #5
    sta RectH
    lda #3
    sta VelocityX
    lda #3
    sta VelocityY
; ----------------------------------------

loop:
    lda #SCREEN_COMMAND_CLEAR
    sta SCREEN_COMMAND

    lda RectX
    sta param1
    lda RectY
    sta param2
    lda RectW
    sta param3
    lda RectH
    sta param4

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

; const param1 - x
; const param2 - y
; const param3 - width
; const param4 - height
; draw_rect: Draws a rectangle on the screen
draw_rect:
    pha

    lda param1 ; x
    sta DrawX
; loop_x: Loops through the width of the rectangle
loop_x:
    sec
    sbc param1 ; x
    cmp param3 ; width
    bcs end_loop_x

    lda param2 ; y
    sta DrawY
; loop_y: Loops through the height of the rectangle
loop_y:
    sec
    sbc param2 ; y
    cmp param4 ; height
    bcs end_loop_y

    jsr putpixel ; Draws a pixel at the current x, y position

    inc DrawY
    lda DrawY
    jmp loop_y 
end_loop_y:
    
    inc DrawX 
    lda DrawX
    jmp loop_x 
end_loop_x:

    pla
    rts ; Returns from the subroutine
; ----------------------------------------

; mut param1 - current position
; mut param2 - velocity
; const param3 - size
; const param4 - max value

advance_and_check:
    pha
    lda param1 ; current position
    clc
    adc param2 ; velocity
    sta param1

    ; check if we hit the edge
    adc param3 ; rectangle dimension (width or height)
    cmp param4 ; screen limit (WIDTH or HEIGHT)
    bcc check_pos ; if we didn't hit the edge, check if we went below zero. otherwise, fix the position

fix_pos:
    sec
    lda #0
    sbc param2 ; negate the velocity
    sta param2 ; store the negated velocity
    adc param1 ; add the negated velocity to the current position
    sta param1 ; store the new position
    jmp end

check_pos:
    ; since positions are unsigned, we can't just check if it's less than 0
    ; instead, we check if it's greater than the max value, and if it is, fix it
    lda param4 ; screen limit (WIDTH or HEIGHT)
    cmp param1
    bcc fix_pos

end:
    pla
    rts
; ----------------------------------------

advance_rect:
    pha

    ; advance x position and check for collisions
    lda RectX
    sta param1
    lda VelocityX
    sta param2
    lda RectW
    sta param3
    lda #WIDTH
    sta param4

    jsr advance_and_check

    lda param1
    sta RectX
    lda param2
    sta VelocityX

    ; advance y position and check for collisions
    lda RectY
    sta param1
    lda VelocityY
    sta param2
    lda RectH
    sta param3
    lda #HEIGHT
    sta param4

    jsr advance_and_check

    lda param1
    sta RectY
    lda param2
    sta VelocityY

    pla
    rts
; ----------------------------------------

	.org $fffc
	.word reset
	.word $0000