import segno

qr_code = segno.make_qr("https://rust-wellcome.github.io/labwhere/")
qr_code.save("qr.png", scale=200)
