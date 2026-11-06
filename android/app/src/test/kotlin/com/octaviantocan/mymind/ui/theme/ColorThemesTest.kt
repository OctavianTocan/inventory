package com.octaviantocan.mymind.ui.theme

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

/** Pully's colour themes in ThyMind: ThyMind's own by default, then Pully's, then every OpenCode theme Pully has. */
class ColorThemesTest {
    /** Looks whose text is under 4.5:1 on their screen as OpenCode draws them, kept rather than repainted (as Pully). */
    private val lowContrast = setOf("solarized/light")

    private fun looks() = ColorThemes.all.flatMap { theme -> listOf("${theme.id}/light" to theme.palette(false), "${theme.id}/dark" to theme.palette(true)) }

    private fun contrast(x: Color, y: Color): Float {
        val (hi, lo) = listOf(x.luminance(), y.luminance()).sortedDescending()
        return (hi + 0.05f) / (lo + 0.05f)
    }

    @Test
    fun thyMindIsTheDefaultAndDrawsExactlyAsBefore() {
        assertSame(ColorThemes.ThyMind, ColorThemes.all.first())
        assertEquals(ColorThemes.DEFAULT, ColorThemes.ThyMind.id)
        assertSame(ColorThemes.ThyMind, ColorThemes.of(null))
        assertSame(ColorThemes.ThyMind, ColorThemes.of("a-theme-that-was-dropped"))
        val light = ColorThemes.ThyMind.palette(false)
        val dark = ColorThemes.ThyMind.palette(true)
        assertSame(MymindLightColors, light.colors)
        assertSame(LightPageColors, light.page)
        assertSame(LightLibraryColors, light.library)
        assertSame(LightCardSheetColors, light.sheet)
        assertSame(MymindDarkColors, dark.colors)
        assertSame(DarkPageColors, dark.page)
        assertSame(DarkLibraryColors, dark.library)
        assertSame(DarkCardSheetColors, dark.sheet)
    }

    @Test
    fun everyPullyThemeIsListedOnceWithAName() {
        val ids = ColorThemes.all.map { it.id }
        assertEquals(ids.distinct(), ids)
        assertEquals("Pully", ColorThemes.all[1].name)
        // Pully offers its own palette and 32 converted from OpenCode; ThyMind adds its own in front and 8 backdrop themes.
        assertEquals(42, ColorThemes.all.size)
        ColorThemes.all.forEach { assertTrue("${it.id} has no name", it.name.isNotBlank()) }
        assertSame(ColorThemes.of("tokyonight"), ColorThemes.all.single { it.name == "Tokyo Night" })
    }

    @Test
    fun aThemeWithOneLookUsesItForBoth() {
        val aura = ColorThemes.of("aura")
        assertSame(aura.palette(false), aura.palette(true))
        assertTrue("Aura is dark in the light look too", aura.palette(false).dark)
    }

    @Test
    fun darkSaysWhichLookAPaletteIs() {
        looks().forEach { (look, p) -> assertEquals(look, p.page.background.luminance() < p.page.text.luminance(), p.dark) }
    }

    @Test
    fun aThemeRepaintsTheGalleryThePagesTheChromeAndTheSheets() {
        val tokyo = ColorThemes.of("tokyonight").palette(true)
        val base = tokyo.dots.first()
        assertEquals(base, tokyo.colors.page)
        assertEquals(base, tokyo.page.background)
        assertEquals(base, tokyo.library.background)
        assertNotEquals(DarkCardSheetColors.paper, tokyo.sheet.paper)
        assertEquals(tokyo.colors.accent, tokyo.library.accent)
    }

    @Test
    fun surfacesAndInkAreOpaque() {
        looks().forEach { (look, p) ->
            listOf(p.colors.page, p.colors.accent, p.page.surface, p.page.text, p.library.raised, p.library.text, p.sheet.paper, p.sheet.text)
                .forEach { assertEquals("$look has a see-through surface or ink", 1f, it.alpha) }
        }
    }

    @Test
    fun textReadsOnTheScreenInBothLooks() {
        val low = looks().filter { (_, p) -> contrast(p.page.text, p.page.background) < 4.5f }.map { it.first }.toSet()
        assertEquals("text under 4.5:1 on the screen", lowContrast, low)
        looks().forEach { (look, p) ->
            val floor = if (look in lowContrast) 3f else 4f
            assertTrue("$look sheet text is at ${contrast(p.sheet.text, p.sheet.paper)}:1", contrast(p.sheet.text, p.sheet.paper) >= floor)
        }
    }

    @Test
    fun theBackdropThemesPaintGradientsAndDoodlesAndSitAfterPully() {
        val ids = ColorThemes.all.map { it.id }
        val made = listOf("sunrise", "ocean", "lavender", "aurora", "notebook", "garden", "night-sky", "postcard")
        assertEquals(made, ids.subList(2, 10))
        made.forEach { id ->
            listOf(false, true).forEach { dark ->
                val backdrop = ColorThemes.of(id).palette(dark).backdrop
                assertTrue("$id has no backdrop", backdrop != null)
                assertTrue("$id doodles must stay faint", backdrop!!.doodles == null || backdrop.ink.alpha <= 0.15f)
            }
        }
        assertEquals(listOf("notebook", "garden", "night-sky", "postcard"), made.filter { ColorThemes.of(it).palette(false).backdrop!!.doodles != null })
        assertTrue("other themes keep a flat screen", ColorThemes.all.filter { it.id !in made }.all { it.palette(false).backdrop == null && it.palette(true).backdrop == null })
    }

    @Test
    fun everyDoodleIsADrawablePath() {
        Doodles.all.forEach { set ->
            assertTrue(set.name, set.shapes.size >= 5)
            set.shapes.forEach { shape -> assertTrue("${set.name}: $shape", androidx.compose.ui.graphics.vector.PathParser().parsePathString(shape).toNodes().isNotEmpty()) }
        }
    }
}
