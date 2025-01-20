# game of colors

a command line implementation of conways game of life where the algorithm was extended to work with colors

```
game_of_colors -i {input image} -o {output directory}
```

optionally you can add:
* -g u32 to determine the number of generations it will simulate
* -threshold f64 to set all pixels in the input below the threshold to black
* -clamp_min f64 or -clamp_max f64 to clamp the input image

when run without an input image the program will generate noise as a starting point. the dimentions of the image can be set with:
* -x u32
* -y u32

