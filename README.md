This code is a hacky attempt to parse the dot file produced by TLA+ and work out the variables changed in state transitions. The names of the changed variables are added to the corresponding links in the dot file (modifying it in place).

`Usage: ./dot_processor <states.dot>`
