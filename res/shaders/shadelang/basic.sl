
in Vec3 normal

Vec3 main() {
    Vec2Test = Vec2(1.0, 1.0)
    Vec3Test = Vec3(0.0, 1.0, 3.0)
    Vec4Test = Vec4(1.0, 4.0, 3.2, 3.1)

    VecAddTest = Vec2Test + Vec2Test
    VecSubTest = Vec3Test - Vec3Test
    VecDivTest = Vec4Test / 2.0
    VecMulTest = Vec4Test * 3.0

    Mat2Test = Mat2(
        1.0, 0.0, 
        0.0, 1.0
    )
    Mat2VecTest = Mat2(
        Vec2(1.0, 0.0),
        Vec2(0.0, 1.0)
    )
    Mat3Test = Mat3(
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0
    )
    Mat3VecTest = Mat3(
        Vec3(1.0, 0.0, 0.0),
        Vec3(0.0, 1.0, 0.0),
        Vec3(0.0, 0.0, 1.0)
    )

    NormalizeTest = normalize(Vec2Test)
    DotTest = dot(Vec3Test, Vec3Test)
    
    L = normalize(Vec3(-0.5, 1.0, -1.0))
    C = Vec3(1.0, 0.5, 0.5)

    cos_a = dot(L, normal)
    ambient = 0.3

    return cos_a * C + ambient * C
}