extends Node2D

## Seam plate for the approved harbour tiles. Not a harbour layout and not
## part of the first playable. Every sprite is placed so its MANIFEST anchor
## lands on the cell bottom plus the shared layer offset (0, +48).

const STEP_U := Vector2(128, 64)
const STEP_V := Vector2(-128, 64)
const LAYER := Vector2(0, 48)
const WATER_ANCHOR := Vector2(128, 127)
const WORKS_ANCHOR := Vector2(128, 255)

func _ready() -> void:
	y_sort_enabled = true
	var origin := Vector2(640, 360)
	# Neighbors share edges with the quay and the pier.
	_water(origin, Vector2i(0, 0), "res://assets/landing/ground/water_b.png")
	_water(origin, Vector2i(1, 0), "res://assets/landing/ground/water_a.png")
	_water(origin, Vector2i(0, 1), "res://assets/landing/ground/water_c.png")
	_water(origin, Vector2i(1, 1), "res://assets/landing/ground/water_a.png")
	_water(origin, Vector2i(-1, 0), "res://assets/landing/ground/water_c.png")
	_water(origin, Vector2i(0, -1), "res://assets/landing/ground/water_b.png")
	_works(origin, Vector2i(0, 1), "res://assets/landing/props/pier_pilings_1x1/beauty.png", 1)
	_works(origin, Vector2i(1, 0), "res://assets/landing/structures/pier_UL_UR_DR_DL/beauty.png", 2)
	_works(origin, Vector2i(0, 0), "res://assets/landing/structures/quay_1111/beauty.png", 2)

	var note := Label.new()
	note.text = "Harbour seam check. Water, quay_1111, pier, pilings. Datum +48. Not the playable."
	note.position = Vector2(24, 16)
	note.add_theme_font_size_override("font_size", 18)
	note.add_theme_color_override("font_color", Color(0.93, 0.9, 0.84))
	add_child(note)

	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var image := get_viewport().get_texture().get_image()
	var path := OS.get_environment("PORTLIGHT_SHOT")
	if path.is_empty():
		path = "/tmp/portlight-harbour-seam.png"
	if image == null or image.is_empty():
		push_error("harbour seam viewport image was empty")
		get_tree().quit(1)
		return
	image.save_png(path)
	print("harbour seam saved ", path)
	get_tree().quit(0)

func _sit(origin: Vector2, cell: Vector2i) -> Vector2:
	return origin + STEP_U * float(cell.x) + STEP_V * float(cell.y) + LAYER

func _water(origin: Vector2, cell: Vector2i, path: String) -> void:
	_sprite(_sit(origin, cell), path, WATER_ANCHOR, 0)

func _works(origin: Vector2, cell: Vector2i, path: String, z: int) -> void:
	_sprite(_sit(origin, cell), path, WORKS_ANCHOR, z)

func _sprite(sit: Vector2, path: String, anchor: Vector2, z: int) -> void:
	var sprite := Sprite2D.new()
	sprite.texture = load(path)
	sprite.centered = false
	sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	sprite.position = sit
	sprite.offset = -anchor
	sprite.z_index = z
	add_child(sprite)
