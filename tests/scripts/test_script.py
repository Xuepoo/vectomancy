import os
from PIL import Image, ImageDraw

# Create a simple black circle image
img = Image.new('RGB', (100, 100), color = 'white')
d = ImageDraw.Draw(img)
d.ellipse((20, 20, 80, 80), fill=(0, 0, 0))

# Save relative to this script's directory
output_dir = os.path.join(os.path.dirname(__file__), '../data')
os.makedirs(output_dir, exist_ok=True)
img.save(os.path.join(output_dir, 'circle.png'))
