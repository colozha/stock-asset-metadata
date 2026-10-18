# Test assets

These are small synthetic assets created for this project, with no user artwork or personal metadata. `artwork.svg` includes legacy Dublin Core metadata and an artwork sentinel. `artwork.eps` is an ASCII PostScript rectangle. `clip.mp4` is a 32×32 solid-color H.264 video (0.2 seconds), generated with FFmpeg; `clip.mov` remuxes that stream into a QuickTime container. Rust tests generate JPEG, PNG, TIFF previews, and binary DOS EPS variants in temporary directories. Neither FFmpeg nor Ghostscript is an application runtime dependency.
