# Test assets

These are small synthetic assets created for this project, with no user artwork or personal metadata. `artwork.svg` includes vector-pro-style legacy Dublin Core metadata and an artwork sentinel. `artwork.eps` is an ASCII PostScript rectangle. `clip.mp4` is a 32×32 solid-color H.264 video (0.2 seconds), generated with FFmpeg; `clip.mov` remuxes that stream into a QuickTime container. Rust tests generate JPEG, PNG, TIFF previews, and binary DOS EPS variants in temporary directories. Neither FFmpeg nor Ghostscript is an application runtime dependency.

Optional local Illustrator samples are read from the `VECTOR_PRO_EPS_DIR` directory, copied to a temporary directory, and never modified in place. They are deliberately not included in version control.
