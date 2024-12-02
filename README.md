Translates DS4 controllers accelerator moition sensors into left thumb stick X movement.

Basically you can tilt the controller like you would be steering in mario kart etc

This is Linux only as it uses the uinput kernal module
 **Tweaks**
 At the top the file there is a variable that allows you to modify the triggers input value scales. My controller is old so my right trigger is fully pressed, it only gets to around 231/255. To fix this I multiply it by 1.1. If you don't have these issues, change it from 1.1 -> 1.0

**Setup**
Make sure the uinput is enabled

```sudo modprobe uinput```

Then I had to add some custom rules so the virtual gamepad is recognised as a joystick.
`sudo nano /etc/udev/rules.d/99-virtual-gamepad.rules`

`KERNEL=="event*", SUBSYSTEM=="input", ATTRS{name}=="Virtual Gamepad", TAG+="uaccess", ENV{ID_INPUT_JOYSTICK}="1"`

Then I reloaded the rules
`sudo udevadm control --reload-rules`
`sudo udevadm trigger`

One potential trouble shoot could be adding your user to the events read/write permissions. Replace X with the event number
`sudo setfacl -m u:{USER}:rw /dev/input/eventX`

I checked what event it was using `evtest`. The name of the gamepad is `Virtual Gamepad`

`/dev/input/event21: Sony Interactive Entertainment Wireless Controller`
`/dev/input/event22: Sony Interactive Entertainment Wireless Controller Motion Sensors  `
`/dev/input/event23: Sony Interactive Entertainment Wireless Controller Touchpad  `
`/dev/input/event24: Virtual Gamepad  `
`/dev/input/event3: Logitech Gaming Mouse G502  `
`/dev/input/event4: Logitech Gaming Mouse G502 Keyboard  `
`/dev/input/event5: USB Keyboard  `
`/dev/input/event6: USB Keyboard Consumer Control  `
`/dev/input/event7: USB Keyboard System Control  `
`/dev/input/event8: USB Keyboard`

For me, it is event 24 'Virtual Gamepad'