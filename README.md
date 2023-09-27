## Constrained Delaunay Triangulation
This will seperate any mesh system with holes in it into nicely spaced triangles with only a few vertices.

### Source
I mostly used [jailbrokengames code](https://github.com/QThund/ConstrainedDelaunayTriangulation/tree/main) and the referenced paper. 
Also bevy was a heavy inspiration for some data types, but has been stripped to be smaller and more efficient.

# TODO
- [ ] look for edge cases (hole outside of polygon, half/half, bigger than poly, hole in hole, hole overlapping)
- [ ] remove all TODO comments
- [ ] refactor things into functions, so that they can be tested (e.g. swap)
- [ ] only use errors to the outside, that are relevant to the user
- [ ] make the API just the "triangulate" function
- [ ] derive all the important derives
- [ ] test squares at input (they might be the reason for the endless loop)
- [ ] test bigger than first polygon input
- [ ] inline functions
- [ ] check what happens if hole point and polygon point are the same