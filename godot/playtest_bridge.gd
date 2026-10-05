# playtest_bridge.gd — autoload. Speaks ai-playtest's newline JSON protocol.
#
#   godot --headless --path godot -- --playtest-port=7777
#
# The bare `--` is required. Without --playtest-port this node turns its
# process off and costs nothing, so a normal launch is unchanged.
extends Node

var _server := TCPServer.new()
var _peer: StreamPeerTCP = null
var _buf := PackedByteArray()
var _port := 0
var _listening := false

func _ready() -> void:
	# An autoload inherits PAUSABLE. ALWAYS keeps the bridge listening if a
	# later pause protocol stops the tree.
	process_mode = Node.PROCESS_MODE_ALWAYS
	_port = _port_from_args()
	if _port == 0:
		set_process(false)

func _port_from_args() -> int:
	var args := OS.get_cmdline_user_args()
	if args.is_empty():
		args = OS.get_cmdline_args()
	for arg in args:
		if arg.begins_with("--playtest-port="):
			return int(arg.split("=")[1])
	return 0

func _process(_delta: float) -> void:
	# Autoload _ready runs before the main scene. Wait until Game exists, then
	# listen, and print the port only after that. The harness waits on the line.
	if not _listening:
		if _game() == null:
			return
		if _server.listen(_port, "127.0.0.1") != OK:
			push_error("playtest bridge could not listen on %d" % _port)
			set_process(false)
			return
		_listening = true
		print("PLAYTEST_BRIDGE_PORT=%d" % _port)
	if _server.is_connection_available():
		if _peer == null:
			_peer = _server.take_connection()
			_peer.set_no_delay(true)
			_on_connect()
		else:
			# One client. A second connection must fail now, not sit in the
			# accept backlog until the first peer leaves.
			var extra := _server.take_connection()
			extra.set_no_delay(true)
			var busy := JSON.stringify({
				"id": 0,
				"error": {"message": "playtest bridge already has a client"},
			}) + "\n"
			extra.put_data(busy.to_utf8_buffer())
			extra.disconnect_from_host()
	if _peer == null:
		return
	_peer.poll()
	if _peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		_peer = null
		_buf.clear()
		return

	# Buffer bytes and decode only complete lines. A split multi-byte
	# character must not be decoded as its own chunk.
	var available := _peer.get_available_bytes()
	if available > 0:
		var chunk: Array = _peer.get_data(available)
		if chunk[0] == OK:
			_buf.append_array(chunk[1])

	while true:
		var nl := _buf.find(0x0A)
		if nl < 0:
			break
		var line := _buf.slice(0, nl).get_string_from_utf8().strip_edges()
		_buf = _buf.slice(nl + 1)
		if line != "":
			_handle(line)

func _on_connect() -> void:
	# A new client starts at Title. A second seat must not inherit a voyage.
	var game := _game()
	if game != null:
		game.playtest_reset()

func _game() -> Node:
	return get_tree().root.get_node_or_null("Game")

func _handle(line: String) -> void:
	var msg: Variant = JSON.parse_string(line)
	if typeof(msg) != TYPE_DICTIONARY:
		_reply_error(0, "request must be one JSON object per line")
		return
	var id: int = int(msg.get("id", 0))
	var method := String(msg.get("method", ""))
	match method:
		"hello":
			var params: Dictionary = msg.get("params", {})
			if int(params.get("protocol", 0)) != 1:
				_reply_error(id, "protocol must be 1")
				return
			_reply(id, {"protocol": 1, "game": "Portlight", "capabilities": ["reset"]})
		"observe":
			_reply(id, _observation())
		"act":
			var game := _game()
			if game == null:
				_reply(id, _not_ready())
				return
			var err := String(game.playtest_apply(msg.get("params", {})))
			if err != "":
				_reply_error(id, err)
				return
			_reply(id, _observation())
		"reset":
			var game_reset := _game()
			if game_reset == null:
				_reply(id, _not_ready())
				return
			game_reset.playtest_reset()
			_reply(id, _observation())
		"quit":
			var final := _observation()
			final["done"] = true
			final["reason"] = "quit"
			_reply(id, final)
			if _peer != null:
				_peer.disconnect_from_host()
			_server.stop()
			get_tree().quit()
		_:
			_reply_error(id, "unknown method")

func _observation() -> Dictionary:
	var game := _game()
	if game == null:
		return _not_ready()
	# The lens adds the contract strip, the Contracts board cards, and the
	# Contracts active rows (state and text). It reads drawn nodes only.
	return PortlightPlaytestLens.augment(game, game.playtest_observation())

func _not_ready() -> Dictionary:
	return {
		"text": "The game is not ready.",
		"state": {},
		"actions": {"kind": "choice", "options": []},
		"done": false,
		"reason": "timeout",
	}

func _reply(id: int, result: Dictionary) -> void:
	if _peer == null or _peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		return
	_peer.put_data((JSON.stringify({"id": id, "result": result}) + "\n").to_utf8_buffer())

func _reply_error(id: int, message: String) -> void:
	if _peer == null or _peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
		return
	_peer.put_data((JSON.stringify({"id": id, "error": {"message": message}}) + "\n").to_utf8_buffer())
