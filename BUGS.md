On Linux when the runtime starts the whole os micro-freezes






≫ let image = Image.load("/mnt/236d6908-7e41-487e-9bfe-13d327f6f722/rust/actiona4/test/screenshot.png")
≫ image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))
error: TypeError: not a function
 --> script:1:34
  |
1 | image.drawCircle(50, 50, 50, new Color(0, 0, 0, 128))
  |                                  ^^^^^








≫ keyboard.waitForKeys([Key.Q])
[Promise] (hint: call `await keyboard.waitForKeys([Key.Q])`)
≫ await keyboard.waitForKeys([Key.Q]) // Doesn't work on Linux
≫ dfqqqQqqQQ
error: ReferenceError: dfqqqQqqQQ is not defined
 --> script:1:1
  |
1 | dfqqqQqqQQ
  | ^^^^^^^^^^

≫ await keyboard.waitForKeys([Key.q])
error: Error: invalid key name
 --> script:1:7
  |
1 | await keyboard.waitForKeys([Key.q])
  |       ^

≫ await keyboard.waitForKeys([Key.Q])
