# lookup
Rust-based word game solver

Stuck finding a crossword clue, where you've got several letters? Try

    lookup _R_E_T

and it will show you all words that fit your clue. Too many results? Maybe the clue is "pressing"
so you can do:

    lookup -t pressing _R_E_T

and it will show the only answer that matches: "urgent". The `-t` flag means a thesaurus lookup.

Have an anagram to solve? Say the anagram is "beget grand urban", the answer is 11,4 in length,
and you have the letters B_A_______G/___E - you can issue the `-j` (jumble) command:

    lookup begetgrandurban -j -f B_A_______G/___E

and it will write out the letters remaining in a circle to make it easier to see the possible
answer (in this case, "Brandenbug Gate"):

        N U D
      A       E
      R       N
      G
        B T R

      B _ A _ _ _ _ _ _ _ G   _ _ _ E

The optional `-f` flag there allows you to specify the letters you've already found.

There are many other options, including regex searches if you understand them.
Type `lookup -h` to see what's available.