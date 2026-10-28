package com.octaviantocan.mymind.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import com.github.takahirom.roborazzi.captureRoboImage
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Cover
import com.octaviantocan.mymind.data.Suggestion
import com.octaviantocan.mymind.data.Tag
import com.octaviantocan.mymind.ui.card.CardDetail
import com.octaviantocan.mymind.ui.library.FeedState
import com.octaviantocan.mymind.ui.library.NewSpaceContent
import com.octaviantocan.mymind.ui.library.SAME_VIBE_LABEL
import com.octaviantocan.mymind.ui.library.SPACE_COLORS
import com.octaviantocan.mymind.ui.library.SearchFilter
import com.octaviantocan.mymind.ui.library.SpaceListContent
import com.octaviantocan.mymind.ui.library.testCard
import com.octaviantocan.mymind.ui.library.cardAccent
import com.octaviantocan.mymind.ui.menu.AddTagsPage
import com.octaviantocan.mymind.ui.menu.DarkMenuColors
import com.octaviantocan.mymind.ui.menu.LocalCardMenuAccent
import com.octaviantocan.mymind.ui.menu.CardMenuPanel
import com.octaviantocan.mymind.ui.menu.CardSubmenu
import com.octaviantocan.mymind.ui.menu.AllSpacesPage
import com.octaviantocan.mymind.ui.menu.SpaceChoices
import com.octaviantocan.mymind.ui.menu.SpaceSuggestion
import com.octaviantocan.mymind.ui.menu.UndoDeleteBar
import com.octaviantocan.mymind.ui.card.ReaderPart
import com.octaviantocan.mymind.ui.menu.CardImagePage
import com.octaviantocan.mymind.ui.menu.ShareContent
import com.octaviantocan.mymind.ui.menu.NotePage
import com.octaviantocan.mymind.ui.menu.PartContent
import com.octaviantocan.mymind.ui.upload.UPLOAD_STRIP_BOTTOM
import com.octaviantocan.mymind.ui.library.CardSpaceCreationState
import com.octaviantocan.mymind.ui.shell.ShellFrame
import com.octaviantocan.mymind.ui.shell.Tab
import com.octaviantocan.mymind.ui.theme.PageScheme
import com.octaviantocan.mymind.ui.theme.PageTheme
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.ParameterizedRobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

/** The card submenus (S12), the new space modal, the search filter chips and the card's tag chips. */
@RunWith(ParameterizedRobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [36], qualifiers = PHONE_QUALIFIERS, fontScale = PHONE_FONT_SCALE)
class SubmenuScreenshotTest(private val dark: Boolean) {
    @get:Rule
    val compose = createComposeRule()

    private val mode = if (dark) "dark" else "light"

    private fun capture(name: String) = compose.onRoot().captureRoboImage("screenshots/${name}_$mode.png")

    private fun render(content: @Composable () -> Unit) {
        compose.setPhoneContent(dark, content)
        compose.mainClock.advanceTimeBy(SETTLE_MS)
    }

    private fun submenu(
        name: String,
        card: Card = LibrarySamples.grid[1],
        page: CardSubmenu = CardSubmenu.Tags,
        title: String = page.title,
        back: Boolean = true,
        content: @Composable () -> Unit,
    ) {
        render {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(
                    Tab.Everything,
                    onSelectTab = {},
                    scheme = PageScheme.Dark,
                    modifier = Modifier.blur(8.dp),
                    overlay = {
                        PageTheme(PageScheme.Dark) {
                            CardDetail(card, onClose = {}, onMenu = {}, headerIcons = com.octaviantocan.mymind.ui.card.defaultCardHeaderIcons())
                        }
                    },
                ) { SampleEverything() }
                CardMenuPanel(title, onDismiss = {}, accent = cardAccent(card), onBack = if (back) ({}) else null) { content() }
            }
        }
        capture(name)
    }

    @Test
    fun addTagsEmpty() = submenu("S12_add_tags_empty") {
        AddTagsPage(TAGS, RECENT, onSave = {}, focusOnOpen = false)
    }

    @Test
    fun addTagsRecent() = submenu("S12_add_tags_recent") {
        AddTagsPage(TAGS, RECENT, onSave = {}, initiallyExpanded = true, focusOnOpen = false)
    }

    @Test
    fun addTagsTyped() = submenu("S12_add_tags_typed") {
        AddTagsPage(TAGS, RECENT, onSave = {}, initialText = "re", focusOnOpen = false)
    }

    @Test
    fun addTagsAdded() = submenu("S12_add_tags_added") {
        AddTagsPage(TAGS, RECENT, onSave = {}, initialTags = listOf("recipes", "weekend", "to try"), focusOnOpen = false)
    }

    @Test
    fun addTagsPaleCardAccent() = submenu("S12_add_tags_pale_accent",
        LibrarySamples.video.copy(cover = Cover(width = 1280, height = 720, dominantColor = "#ffe3b5"))) {
        AddTagsPage(TAGS, RECENT, onSave = {}, focusOnOpen = false)
    }

    @Test
    fun selectSpace() = submenu("S12_select_space", page = CardSubmenu.Spaces) {
        val spaces = LibrarySamples.spaces.spaces
        AllSpacesPage(SpaceChoices(spaces, selected = setOf("s2", "s3"), suggestion = SpaceSuggestion(spaces.first(), 3)), CardSpaceCreationState(),
            onToggle = {}, onCreate = { _, _ -> })
    }

    @Test
    fun selectSpaceEmpty() = submenu("S12_select_space_empty", page = CardSubmenu.Spaces) {
        AllSpacesPage(SpaceChoices(emptyList(), selected = emptySet()), CardSpaceCreationState(), onToggle = {}, onCreate = { _, _ -> })
    }

    @Test
    fun pictureSheet() = submenu("S14_part_picture", PICTURED, title = ReaderPart.Picture.heading, back = false) {
        PartContent(ReaderPart.Picture, PICTURED, onChoice = {})
    }

    @Test
    fun noteSheet() = submenu("S14_part_note", PICTURED, title = ReaderPart.YourNote.heading, back = false) {
        PartContent(ReaderPart.YourNote, PICTURED, onChoice = {})
    }

    @Test
    fun writeNote() = submenu("S14_write_note", PICTURED, page = CardSubmenu.Note) {
        NotePage(PICTURED.note.orEmpty(), onSave = {}, focusOnOpen = false)
    }

    @Test
    fun shareSheet() = submenu("S14_share", PICTURED, title = "Share", back = false) {
        ShareContent(PICTURED, onShareLink = {}, onShareText = null, onCopyLink = {}, onSharePicture = {}, onCardImage = {})
    }

    @Test
    fun cardAsImage() = submenu("S14_card_image", PICTURED, page = CardSubmenu.CardImage) {
        CardImagePage(PICTURED, onShare = {})
    }

    private fun newSpace(name: String, step: Int) {
        render {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(Tab.Spaces, onSelectTab = {}, scheme = PageScheme.System) {
                    SpaceListContent(LibrarySamples.spaces, onOpenSpace = {}, onRetry = {})
                }
                PageTheme(PageScheme.System) {
                    Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.8f)), contentAlignment = Alignment.Center) {
                        NewSpaceContent(
                            onCreate = { _, _ -> },
                            initialName = if (step == 1) "" else "Weekend plans",
                            initialStep = step,
                            initialColor = if (step == 1) null else SPACE_COLORS[9],
                            focusOnOpen = false,
                        )
                    }
                }
            }
        }
        capture(name)
    }

    @Test
    fun newSpaceName() = newSpace("S07_new_space_name", 1)

    @Test
    fun newSpaceColor() = newSpace("S07_new_space_color", 2)

    @Test
    fun undoDelete() {
        render {
            Box(Modifier.fillMaxSize()) {
                ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.Dark) { SampleEverything() }
                UndoDeleteBar(LibrarySamples.article.title.orEmpty(), onUndo = {},
                    Modifier.align(Alignment.BottomCenter).padding(start = 16.dp, end = 16.dp, bottom = UPLOAD_STRIP_BOTTOM))
            }
        }
        capture("S13_undo_delete")
    }

    private fun shell(content: @Composable () -> Unit) =
        render { ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.Dark, page = content) }

    @Test
    fun searchTagFilter() {
        shell {
            SampleEverything(
                searching = true,
                filter = SearchFilter("recipes", "tag:recipes"),
                suggestions = listOf(Suggestion("tag", "Reading", "tag:Reading"), Suggestion("tag", "references", "tag:references")),
                results = FeedState(LibrarySamples.grid.take(5), endReached = true),
            )
        }
        capture("S09_search_tag_filter")
    }

    @Test
    fun searchSameVibe() {
        shell {
            SampleEverything(searching = true, filter = SearchFilter(SAME_VIBE_LABEL, "similar:g7"), results = FeedState(LibrarySamples.grid.drop(5), endReached = true))
        }
        capture("S09_search_same_vibe")
    }

    @Test
    fun cardTags() {
        val card = testCard("k1", type = "Article", url = "https://example.com/k1", title = "A card with tags")
            .copy(description = "A short description.", tags = listOf(Tag("t1", "recipes"), Tag("t2", "weekend"), Tag("t3", "to try")))
        render {
            ShellFrame(Tab.Everything, onSelectTab = {}, scheme = PageScheme.Dark, overlay = {
                PageTheme(PageScheme.Dark) {
                    CardDetail(card, onClose = {}, onMenu = {}, headerIcons = com.octaviantocan.mymind.ui.card.defaultCardHeaderIcons(), onRemoveTag = {})
                }
            }) { SampleEverything() }
        }
        capture("S10_card_tags")
    }

    companion object {
        private const val SETTLE_MS = 1_500L
        private val TAGS = listOf("recipes", "reading", "references", "travel", "design", "research", "weekend", "red", "pre-order")
        private val RECENT = listOf("weekend", "design", "recipes")
        private val PICTURED = testCard("a1", type = "Article", url = "https://www.theatlantic.com/walking", title = "Walking",
            tags = listOf(Tag("t1", "walks"), Tag("t2", "essays"), Tag("t3", "nature")))
            .copy(description = "An essay on walking.", note = "Reread the part about walking west.",
                cover = Cover(src = "/media/walking.jpg", width = 1600, height = 1067, dominantColor = "#c9b79c"))

        /** System light and system dark. */
        @JvmStatic
        @ParameterizedRobolectricTestRunner.Parameters(name = "dark={0}")
        fun modes() = listOf(arrayOf<Any>(false), arrayOf<Any>(true))
    }
}
