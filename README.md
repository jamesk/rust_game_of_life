# Concurrent Conway's Game of Life
I wanted to play around in [Rust](https://www.rust-lang.org/), as my first project in the language I wanted something smallish. So I thought I'd make an implementation of [Conway's Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life). However to really get into the interesting parts of Rust I decided to aim for having the processing be concurrent/distributed.

#Overview
_this is likely to get outdated as I restructure the project_

Current concept / architecture is that the grid or board is seperated into sections. Each section contains multiple cells. The sections communicate via message passing (currently using Rust's built in channels). The view is also rendered based on updates passed through channels.

A collection of threads is made and given ownership of a subset of the sections making up the board. These threads are very simple and just loop, continuously calling a function on the sections to trigger processing.

An interesting area I've explored a little is that if one section is slow/fails to communicate, this doesn't neccesitate the whole simulation stopping. Indeed each successive generation of cells can be calculated with only the knowledge of its neighbours. So there is a propogation of information across the grid at a rate of 1 cell/space per generation. Hence if a section fails, distant parts of the grid can continue to function and calculate their next generations with no worries until the lack of information has propogated across the board to reach them.

#Interface
The interface is a window where a 2D grid is rendered to represent the state of the simulation. An alive cell is represented as a green square while a dead cell is represented as a white square. Another dimension however is added in this view of the Game of Life, the "age" of the displayed cell. There is an effective global generation that in a perfect system all cells are at. However if a cell has fallen behind (perhaps due to a lack of information from its neighbours or other failure) it will be from an older generation. This is displayed on the interface by darkening the square for that cell, the darker the square the older the cell is. In the event that the interface has no information about a cell its corresponding square on the grid will be completely black.

