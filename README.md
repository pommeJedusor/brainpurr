# brainpurr

## description
brainpurr is a Turing complete programming language made entirely of cat noises: meow, mrow, mrp, purr, :3c, >:3, nya and :3.
it is strongly inspired by [brainfuck](https://en.wikipedia.org/wiki/Brainfuck) (to not say a total copy)

## how it works

the way brainpurr works is that it stores in memory 4 different things:
1. the array of instructions (see list below)
2. a pointer tracking the current position in the instruction array
3. a one dimensional array of 1 byte intergers of unlimited size each being 0 by default (implemented as a vector that grows dynamically as needed)
4. a pointer tracking the current position in the one dimensional array

| Name | Instructions                                                                                                           |
| ---- | ---------------------------------------------------------------------------------------------------------------------- |
| meow | increments the pointer by 1                                                                                            |
| mrow | decrements the pointer by 1                                                                                            |
| mrp  | increments the integer at the index of the pointer by 1                                                                |
| purr | decrements the integer at the index of the pointer by 1                                                                |
| :3c  | outputs the integer at the index of the pointer                                                                        |
| >:3  | replaces the integer at the index of the pointer by an integer input by the user                                       |
| nya  | if the current byte is 0 then move to the character after the matching :3 instead of going to the next instruction     |
| :3   | if the current byte isn't 0 then move to the character after the matching nya instead of going to the next instruction |
