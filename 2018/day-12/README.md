# Solution for part-2

Part-2 expect us to find the sum of pot's number containing plants after *50_000_000_000* iteration.
This is require a lot of time (and resource) to do it as per instruction.
To complete this task, I run the solution for part-1 for 500 iteration, and try to see the pattern there.

Luckily, in my case the pattern is forming after 194 generation.
On that generation and above, the patterns of pots contains plant is fixed, with the difference between each generation is increment one to the left pos (and the right pos).
In term of calculation, from that point onward, each generation only differ by constant margin, thus makes it possible to calculate the result on 50_000_000_000th generation.
