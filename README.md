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

## Dependencies

The tests require the nerd fonts to be installed. Run the
following:

### Windows
```powershell
scoop bucket add extras
scoop bucket add nerd-fonts 
scoop install Monaspace-NF FiraCode-NF ProFont-NF CascadiaCode-NF Noto-NF
```

### Mac
```bash
brew install font-monaspace font-fira-code-nerd-font font-profont-nerd-font font-caskaydia-cove-nerd-font font-monaspace-nerd-font font-noto-nerd-font
```
### Linux
```bash
FONT_DIR="${HOME}/.local/share/fonts"
mkdir -p "$FONT_DIR"

for font in ${{ env.FONTS }}; do
  ZIP_FILE="${font}${EXTENSION}"
  if [[ "$font" == "Monaspace" ]]; then
    DOWNLOAD_URL="https://github.com/githubnext/monaspace/releases/download/v1.101/monaspace-v1.101.zip"
  else
    DOWNLOAD_URL="https://github.com/ryanoasis/nerd-fonts/releases/download/${VERSION}/${ZIP_FILE}"
  fi
  echo "Downloading and installing '$font'..."
  wget --quiet "$DOWNLOAD_URL" -O "$ZIP_FILE"
  unzip -oq "$ZIP_FILE" -d "$FONT_DIR"
  rm "$ZIP_FILE"
  echo "'$font' installed successfully."
done

# Refresh font cache
fc-cache -fv
```
