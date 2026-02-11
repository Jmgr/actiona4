- run eval "" displays an update check error

≫ await screenshot.captureRect(0, 0, 100, 100).save("out.png")
error: TypeError: not a function
 --> script:1:41
  |
1 | await screenshot.captureRect(0, 0, 100, 100).save("out.png")
  |                                         ^

≫ await (await screenshot.captureRect(0, 0, 100, 100)).save("out.png")

- screencapture with a rect produce a dark grey area? Zed background?
