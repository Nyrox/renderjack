

in float nx
in float ny
in float nz

out float cr
out float cg
out float cb

void main() {
    lx = -0.5
    ly = 1.0
    lz = -1.0

    cos_a = lx * nx + ly * ny + lz * nz
    cr = cos_a * 1.0
    cg = cos_a * 0.2
    cb = cos_a * 0.2
}