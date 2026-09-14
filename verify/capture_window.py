"""Captures a window's own content (no desktop behind it) with rounded corners.

usage: python capture_window.py "<window title>" out.png

Uses PrintWindow(PW_RENDERFULLCONTENT) so WebView2 content is included, crops to
the DWM frame bounds (drops the invisible shadow margin) and makes the corners
transparent like Windows 11.
"""
import ctypes
import ctypes.wintypes as w
import sys

from PIL import Image, ImageDraw

u = ctypes.windll.user32
g = ctypes.windll.gdi32
dwm = ctypes.windll.dwmapi
u.SetProcessDPIAware()

PW_RENDERFULLCONTENT = 2
DWMWA_EXTENDED_FRAME_BOUNDS = 9
CORNER_RADIUS = 8


class BITMAPINFOHEADER(ctypes.Structure):
    _fields_ = [
        ("biSize", w.DWORD), ("biWidth", w.LONG), ("biHeight", w.LONG), ("biPlanes", w.WORD),
        ("biBitCount", w.WORD), ("biCompression", w.DWORD), ("biSizeImage", w.DWORD),
        ("biXPelsPerMeter", w.LONG), ("biYPelsPerMeter", w.LONG), ("biClrUsed", w.DWORD), ("biClrImportant", w.DWORD),
    ]


def capture(title: str, out: str) -> None:
    hwnd = u.FindWindowW(None, title)
    if not hwnd:
        sys.exit(f"window not found: {title}")

    win = w.RECT()
    u.GetWindowRect(hwnd, ctypes.byref(win))
    frame = w.RECT()
    if dwm.DwmGetWindowAttribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, ctypes.byref(frame), ctypes.sizeof(frame)) != 0:
        frame = win
    width, height = win.right - win.left, win.bottom - win.top

    hdc_window = u.GetWindowDC(hwnd)
    hdc_mem = g.CreateCompatibleDC(hdc_window)
    bitmap = g.CreateCompatibleBitmap(hdc_window, width, height)
    g.SelectObject(hdc_mem, bitmap)
    u.PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT)

    header = BITMAPINFOHEADER()
    header.biSize = ctypes.sizeof(BITMAPINFOHEADER)
    header.biWidth, header.biHeight = width, -height  # top-down
    header.biPlanes, header.biBitCount = 1, 32
    buf = ctypes.create_string_buffer(width * height * 4)
    g.GetDIBits(hdc_mem, bitmap, 0, height, buf, ctypes.byref(header), 0)

    g.DeleteObject(bitmap)
    g.DeleteDC(hdc_mem)
    u.ReleaseDC(hwnd, hdc_window)

    image = Image.frombuffer("RGBA", (width, height), buf, "raw", "BGRA", 0, 1).convert("RGB")
    left, top = frame.left - win.left, frame.top - win.top
    image = image.crop((left, top, left + frame.right - frame.left, top + frame.bottom - frame.top)).convert("RGBA")

    mask = Image.new("L", image.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, image.width - 1, image.height - 1), radius=CORNER_RADIUS, fill=255)
    image.putalpha(mask)
    image.save(out)
    print(f"saved {out} {image.size}")


if __name__ == "__main__":
    capture(sys.argv[1], sys.argv[2])
