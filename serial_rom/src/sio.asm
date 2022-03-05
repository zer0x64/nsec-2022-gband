section "SIO Code", rom0

include "hardware.inc"
include "sio.inc"

init_sio_slave::
    ld a, SLAVE_MODE
	ld [rSB], a
	ld a, SCF_START
	ld [rSC], a
    ret

init_sio_master::
    ld a, MASTER_MODE
	ld [rSB], a
	ld a, SCF_START | SCF_SOURCE
	ld [rSC], a
    ret

sio_slave_transfer::
	ld [rSB], a
	ld a, SCF_START
	ld [rSC], a
    ret

sio_master_transfer::
	ld [rSB], a
	ld a, SCF_START | SCF_SOURCE
	ld [rSC], a
    ret
