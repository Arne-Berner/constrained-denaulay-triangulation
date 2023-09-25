## Constrained Delaunay Triangulation
This will seperate any mesh system with holes in it into nicely spaced triangles with only a few vertices.

### Source
I mostly used [jailbrokengames code](https://github.com/QThund/ConstrainedDelaunayTriangulation/tree/main) and the referenced paper. 
Also bevy was a heavy inspiration for some data types, but has been stripped to be smaller and more efficient.

# TODO
- [ ] remove all TODO comments
- [ ] only use errors to the outside, that are relevant to the user
- [ ] make the API just the "triangulate" function
- [ ] derive all the important derives