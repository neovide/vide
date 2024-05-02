<div align="center">

# Vide

</div>

A basic wgpu gui renderer intended to be used with
[Neovide](https://neovide.dev/) based on ideas from Vello
and Zed's rendering approaches but with a focus on
simplicity.

Some key goals and principles we are trying to follow with this
effort:

1. Use wgpu as the graphics library for maximum portability.
2. Define a scene format which can fluently be defined in
   rust code, and can be serialized to disk for easy
   rendering and logging.
3. Render the entire scene layer by layer. Within a layer
   individual components are drawn in declaration order by
   kind reducing the need for offscreen buffers.
4. Where possible, features of the renderer should be
   regression tested to ensure that changes do not change
   the quality of the rendering unless explicitly intended.
