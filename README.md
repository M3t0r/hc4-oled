# HC4 OLED

A small utility that collects and displays some data from the host, like free
disk space and CPU load. It has a very basic OLED burn-in avoidance system.

![A photo of a HC4 OLED display showing the program output](./example.avif)

The monochrome OLED display on HC4s is connected via I2C, using a SSD1306
driver chip. There are some more informations and a Python code example in
Odroid's [HC4 application notes](https://wiki.odroid.com/odroid-hc4/application_note/oled).
