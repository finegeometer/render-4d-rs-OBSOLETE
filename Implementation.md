A summary of how this crate works, as of 2019-09-22.

# Note

Sometimes when I say ℝℙ^n, I may actually mean it's double cover, S^n. I've forgotten where the sign matters.

# `Mesh`

A `Mesh` represents the geometry of the world. It consists of a collection of `Facet`s.

A `Facet` represents a piece of the geometry of a world that lies within a single 3-surface. It consists of:
- An embedding of ℝℙ^3 into ℝℙ^4. The image of this embedding is this `Facet`'s 3-surface.
- A collection of convex regions of this 3-surface, whose union is the solid part of this piece of geometry.
- A collection of `Texture`s, representing the visible part of this piece of geometry.

A `Texture` represents a visible 2-surface in the geometry. It consists of:
- An embedding of ℝℙ^2 into ℝℙ^3. The image of this embedding is this `Texture`'s 2-surface. The preimage of a point on said surface is it's texture coordinates.
- A `Polygon`, from my `polygon3` crate. This is the boundary of the visible 2-surface.

# `Triangle`

A `Triangle` represents a trianglular face in the result of the projection. It consists of:
- Three vertices, of type `Vertex`.
- A boolean flag. If the flag is true, the triangle should be subtracted from the picture, rather than added to it. (Triangulation of polygons is trivial when negative triangles are allowed.)

A `Vertex` consists of a position in ℝℙ^3, and a texture coordinate in ℝℙ^2.

# `Mesh::project`

The function `Mesh::project` takes a `Mesh` and a projection matrix (ℝℙ^4 -> ℝℙ^4).
It removes the parts of each `Texture` in the geometry that are hidden behind other `Facet`s.
Then it triangulates all of the textures, and spits out the result as a bunch of `Triangle`s.