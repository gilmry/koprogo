# PWA Icons

This directory should contain PWA icons in the following sizes:

- **icon-72x72.png** - For small screens and badges
- **icon-96x96.png** - For standard mobile devices
- **icon-128x128.png** - For high-resolution mobile devices
- **icon-144x144.png** - For Windows tablets
- **icon-152x152.png** - For iOS devices
- **icon-192x192.png** - For Android devices (minimum)
- **icon-384x384.png** - For high-resolution Android devices
- **icon-512x512.png** - For Android devices and splash screens (required)

## How to Generate Icons

You can use online tools or command-line tools to generate all sizes from a single source image:

### Option 1: Online Tool

Use [PWA Asset Generator](https://www.pwabuilder.com/imageGenerator) or [Real Favicon Generator](https://realfavicongenerator.net/)

### Option 2: ImageMagick (Command Line)

```bash
# Install ImageMagick if not already installed
# Ubuntu/Debian: sudo apt-get install imagemagick
# macOS: brew install imagemagick

# Generate all sizes from source.png
convert source.png -resize 72x72 icon-72x72.png
convert source.png -resize 96x96 icon-96x96.png
convert source.png -resize 128x128 icon-128x128.png
convert source.png -resize 144x144 icon-144x144.png
convert source.png -resize 152x152 icon-152x152.png
convert source.png -resize 192x192 icon-192x192.png
convert source.png -resize 384x384 icon-384x384.png
convert source.png -resize 512x512 icon-512x512.png
```

### Option 3: Sharp (Node.js)

```bash
npm install -g sharp-cli

sharp -i source.png -o icon-72x72.png resize 72 72
sharp -i source.png -o icon-96x96.png resize 96 96
sharp -i source.png -o icon-128x128.png resize 128 128
sharp -i source.png -o icon-144x144.png resize 144 144
sharp -i source.png -o icon-152x152.png resize 152 152
sharp -i source.png -o icon-192x192.png resize 192 192
sharp -i source.png -o icon-384x384.png resize 384 384
sharp -i source.png -o icon-512x512.png resize 512 512
```

## Design Guidelines

- Use a **square** image (1:1 aspect ratio)
- Recommended source size: **1024x1024px** or larger
- Use **transparent background** for maskable icons
- Include **safe zone** (80% of icon area) for content
- Test on both **light and dark** backgrounds
- Follow [Material Design icon guidelines](https://material.io/design/iconography/)

## Temporary Placeholder

Until you provide the actual icons, you can use a placeholder SVG or generate simple colored squares:

```bash
# Generate simple colored placeholders
convert -size 512x512 xc:#10b981 icon-512x512.png
convert -size 384x384 xc:#10b981 icon-384x384.png
convert -size 192x192 xc:#10b981 icon-192x192.png
# ... etc for all sizes
```
