plugins {
    `kotlin-dsl`
}

dependencies {
    testImplementation(libs.junit)
}

tasks.test {
    useJUnit()
}
