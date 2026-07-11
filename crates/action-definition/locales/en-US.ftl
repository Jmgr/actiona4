action-click =
    .name = Click
    .description = Click at a position on the screen.

action-click-position =
    .name = Position
    .description = The screen position to click.

action-click-button =
    .name = Button
    .description = The mouse button to click.

action-click-relative-position =
    .name = Relative position
    .description = Whether the position is relative to the current cursor position.

action-click-amount =
    .name = Amount
    .description = The number of times to click.

action-click-interval =
    .name = Interval
    .description = The delay between consecutive clicks.

action-click-duration =
    .name = Duration
    .description = How long to hold each click.

enum-mouse-button =
    .left = Left
    .middle = Middle
    .right = Right
    .back = Back
    .forward = Forward

action-button-condition =
    .name = Button condition
    .description = Continue based on whether a mouse button is pressed.

action-double-click =
    .name = Double-click
    .description = Double-click at a position on the screen.

action-double-click-delay =
    .name = Delay
    .description = The delay between the two clicks.

action-get-cursor-position =
    .name = Get cursor position
    .description = Store the current mouse cursor position.

action-get-cursor-position-result =
    .name = Result
    .description = The variable that receives the cursor position.

action-message-box =
    .name = Message box
    .description = Show a message box and continue based on the selected button.

action-message-box-title =
    .name = Title
    .description = The title shown in the message box.

action-message-box-text =
    .name = Text
    .description = The message shown in the message box.

action-message-box-buttons =
    .name = Buttons
    .description = The set of buttons shown in the message box.

enum-message-box-buttons =
    .ok = OK
    .ok-cancel = OK / Cancel
    .yes-no = Yes / No
    .yes-no-cancel = Yes / No / Cancel

action-message-box-icon =
    .name = Icon
    .description = The icon shown in the message box.

enum-message-box-icon =
    .info = Info
    .warning = Warning
    .error = Error

action-message-box-ok-label =
    .name = OK label
    .description = The custom label for the OK button.

action-message-box-yes-label =
    .name = Yes label
    .description = The custom label for the Yes button.

action-message-box-no-label =
    .name = No label
    .description = The custom label for the No button.

action-message-box-cancel-label =
    .name = Cancel label
    .description = The custom label for the Cancel button.

action-move-cursor =
    .name = Move cursor
    .description = Move the mouse cursor to a position.

action-move-cursor-position =
    .name = Position
    .description = The screen position to move the cursor to.

action-move-cursor-speed =
    .name = Speed
    .description = The cursor movement speed.

action-move-cursor-tween =
    .name = Tween
    .description = The easing function used for cursor movement.

enum-tween =
    .back-in = Back in
    .back-in-out = Back in/out
    .back-out = Back out
    .bounce-in = Bounce in
    .bounce-in-out = Bounce in/out
    .bounce-out = Bounce out
    .circ-in = Circular in
    .circ-in-out = Circular in/out
    .circ-out = Circular out
    .cubic-in = Cubic in
    .cubic-in-out = Cubic in/out
    .cubic-out = Cubic out
    .elastic-in = Elastic in
    .elastic-in-out = Elastic in/out
    .elastic-out = Elastic out
    .expo-in = Exponential in
    .expo-in-out = Exponential in/out
    .expo-out = Exponential out
    .linear = Linear
    .quad-in = Quadratic in
    .quad-in-out = Quadratic in/out
    .quad-out = Quadratic out
    .quart-in = Quartic in
    .quart-in-out = Quartic in/out
    .quart-out = Quartic out
    .quint-in = Quintic in
    .quint-in-out = Quintic in/out
    .quint-out = Quintic out
    .sine-in = Sine in
    .sine-in-out = Sine in/out
    .sine-out = Sine out

action-move-cursor-perlin-scale =
    .name = Perlin scale
    .description = The scale of cursor movement noise.

action-move-cursor-perlin-amplitude =
    .name = Perlin amplitude
    .description = The amplitude of cursor movement noise.

action-move-cursor-target-randomness =
    .name = Target randomness
    .description = The amount of random variation applied to the target position.

action-move-cursor-interval =
    .name = Interval
    .description = The delay between movement updates.

action-set-cursor-position =
    .name = Set cursor position
    .description = Set the mouse cursor position immediately.

action-set-cursor-position-position =
    .name = Position
    .description = The screen position to set the cursor to.

action-press =
    .name = Press
    .description = Press and hold a mouse button.

action-press-position =
    .name = Position
    .description = The screen position to move to before pressing.

action-press-button =
    .name = Button
    .description = The mouse button to press.

action-release =
    .name = Release
    .description = Release a mouse button.

action-release-button =
    .name = Button
    .description = The mouse button to release. If unset, releases the most recently pressed button.

action-scroll =
    .name = Scroll
    .description = Scroll the mouse wheel.

action-scroll-amount =
    .name = Amount
    .description = The scroll amount.

action-scroll-axis =
    .name = Axis
    .description = The scroll axis.

enum-axis =
    .horizontal = Horizontal
    .vertical = Vertical

action-wait-for-button =
    .name = Wait for button
    .description = Wait until a mouse button is pressed or released.

action-wait-for-button-button =
    .name = Button
    .description = The mouse button to wait for. If unset, waits for any button.

action-wait-for-button-direction =
    .name = Direction
    .description = The button direction to wait for. If unset, waits for either press or release.

enum-button-direction =
    .press = Press
    .release = Release

action-wait-for-movement =
    .name = Wait for movement
    .description = Wait until the mouse cursor moves.

action-wait-for-scroll =
    .name = Wait for scroll
    .description = Wait until the mouse wheel is scrolled.

action-wait-for-scroll-axis =
    .name = Axis
    .description = The scroll axis to wait for. If unset, waits for any axis.

action-clear-clipboard =
    .name = Clear clipboard
    .description = Clear the clipboard contents.

action-clear-clipboard-selection =
    .name = Selection
    .description = Whether to clear the Linux selection clipboard.

action-get-clipboard-text =
    .name = Get clipboard text
    .description = Store the current clipboard text.

action-get-clipboard-text-result =
    .name = Result
    .description = The variable that receives the clipboard text.

action-get-clipboard-text-selection =
    .name = Selection
    .description = Whether to read from the Linux selection clipboard.

action-set-clipboard-text =
    .name = Set clipboard text
    .description = Set the clipboard text.

action-set-clipboard-text-text =
    .name = Text
    .description = The text to write to the clipboard.

action-set-clipboard-text-selection =
    .name = Selection
    .description = Whether to write to the Linux selection clipboard.

action-wait-for-clipboard-changed =
    .name = Wait for clipboard changed
    .description = Wait until the clipboard contents change.

action-wait-for-clipboard-changed-check-interval =
    .name = Check interval
    .description = The delay between clipboard checks.

action-wait-for-clipboard-changed-selection =
    .name = Selection
    .description = Whether to watch the Linux selection clipboard.

action-code =
    .name = Code
    .description = Run source code and continue through any named branches it defines.

action-code-source =
    .name = Source
    .description = The source code to run.

action-test =
    .name = Test
    .description = Prototype action for testing editor behavior.

action-test-percent =
    .name = Percent
    .description = The percentage value used by the test action.

action-test-duration =
    .name = Duration
    .description = The duration value used by the test action.

action-goto =
    .name = Go to label
    .description = Continue execution at a labeled action.

action-goto-target =
    .name = Target
    .description = The label to continue execution at.

action-stop =
    .name = Stop
    .description = Stop execution.

action-exit =
    .name = Exit
    .description = Stop execution and exit the application.

action-marker =
    .name = Marker
    .description = Mark a place in the action list that other flow actions can jump to.

action-wait =
    .name = Wait
    .description = Wait for a duration.

action-wait-duration =
    .name = Duration
    .description = The amount of time to wait.

action-wait-unit =
    .name = Unit
    .description = The time unit for the duration.

enum-wait-unit =
    .milliseconds = Milliseconds
    .seconds = Seconds
    .minutes = Minutes
    .hours = Hours
    .days = Days

action-loop =
    .name = Loop
    .description = Run its body a fixed number of times.

action-loop-max-counter =
    .name = Maximum count
    .description = The maximum number of times to run the loop body.

action-and =
    .name = Wait for all
    .description = Wait until every input completes.

action-or =
    .name = Wait for any
    .description = Wait until one input completes.

action-if =
    .name = If
    .description = Branch execution depending on whether a value is true.

action-if-value =
    .name = Value
    .description = The value to evaluate; execution continues through the true or false branch depending on whether it is truthy.

action-switch =
    .name = Switch
    .description = Branch execution depending on which case a value matches.

action-switch-value =
    .name = Value
    .description = The value to compare against each case; execution continues through the matching case branch, or the default branch if none match.

action-timeout =
    .name = Timeout
    .description = The maximum time this action can run before continuing through the timeout branch.
