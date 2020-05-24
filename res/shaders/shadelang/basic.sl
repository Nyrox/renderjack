
in Vec3 normal

Vec3 main() {
    L = normalize(Vec3(-0.5, 1.0, -1.0))
    C = Vec3(1.0, 0.5, 0.5)
    
    cos_a = dot(L, normal)

    return cos_a * C
}