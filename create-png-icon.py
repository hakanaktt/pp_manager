#!/usr/bin/env python3
"""
Simple script to create a basic PNG icon for development
"""
import struct
import zlib

def create_simple_png():
    width, height = 32, 32
    
    # Create pixel data (RGBA)
    pixels = []
    for y in range(height):
        row = []
        for x in range(width):
            # Create a simple 'P' pattern
            if ((x >= 4 and x <= 20 and y == 4) or      # Top line
                (x >= 4 and x <= 16 and y == 16) or     # Middle line  
                (x == 4 and y >= 4 and y <= 28) or      # Left line
                (x == 20 and y >= 4 and y <= 16)):      # Right line (top part)
                # White pixel for 'P'
                row.extend([255, 255, 255, 255])  # RGBA
            else:
                # Blue background
                row.extend([0, 100, 200, 255])     # RGBA
        pixels.extend(row)
    
    # PNG signature
    png_signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)  # 8-bit RGBA
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff
    ihdr_chunk = struct.pack('>I', len(ihdr_data)) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc)
    
    # IDAT chunk
    # Add filter bytes (0 = no filter) for each row
    filtered_data = b''
    for y in range(height):
        filtered_data += b'\x00'  # No filter
        start = y * width * 4
        end = start + width * 4
        filtered_data += bytes(pixels[start:end])
    
    compressed_data = zlib.compress(filtered_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed_data) & 0xffffffff
    idat_chunk = struct.pack('>I', len(compressed_data)) + b'IDAT' + compressed_data + struct.pack('>I', idat_crc)
    
    # IEND chunk
    iend_crc = zlib.crc32(b'IEND') & 0xffffffff
    iend_chunk = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc)
    
    # Combine all chunks
    png_data = png_signature + ihdr_chunk + idat_chunk + iend_chunk
    
    return png_data

if __name__ == "__main__":
    png_data = create_simple_png()
    with open("src-tauri/icons/32x32.png", "wb") as f:
        f.write(png_data)
    print("Created 32x32.png icon")
