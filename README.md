## Constrained Delaunay Triangulation
This will seperate any mesh system with holes in it into nicely spaced triangles with only a few vertices.

### Source
I mostly used [jailbrokengames code](https://github.com/QThund/ConstrainedDelaunayTriangulation/tree/main) and the referenced paper. 
Also bevy was a heavy inspiration for some data types, but has been stripped to be smaller and more efficient.

# TODO
- [ ] refactor things into functions, so that they can be tested (e.g. swap)
- [ ] remove all TODO comments
- [ ] Polygons should not need to be in a specific order
- [ ] look for edge cases (hole outside of polygon, half/half, bigger than poly, hole in hole, hole overlapping)
- [ ] only use errors to the outside, that are relevant to the user
- [ ] make the API just the "triangulate" function
- [ ] derive all the important derives
- [ ] test bigger than first polygon input
- [ ] inline functions
- [ ] check what happens if hole point and polygon point are the same
- [ ] use robust crate for float
- [ ] what happens, if any new point is on the line of another triangle?
- [ ] safe the contour of the point cloud, for an "hole out of bounds" error



### Holes outside of bounds
If the holes are stretched out outside of the point cloud, then they cannot be triangulated, unless we would keep track of all the actual connections to those very nodes. That would make the algorithm much slower.