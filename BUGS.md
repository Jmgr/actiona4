After running the repl for 3 min, this appeared:

2026-02-23T22:41:40.990138Z ERROR rodio::stream: 153: audio stream error: Buffer underrun/overrun occurred.

I was not playing any sound.






≫ let image = Image.load("/mnt/236d6908-7e41-487e-9bfe-13d327f6f722/rust/actiona4/test/screenshot.png")
≫ image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))
error: TypeError: not a function
 --> script:1:34
  |
1 | image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))
  |                                  ^^^^^
