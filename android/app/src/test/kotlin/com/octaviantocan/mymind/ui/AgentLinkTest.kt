package com.octaviantocan.mymind.ui

import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.menu.agentLink
import com.octaviantocan.mymind.ui.menu.spaceAgentLink
import org.junit.Assert.assertEquals
import org.junit.Test

class AgentLinkTest {
    private val card = Card(id = "00000002VideoAAAAAAAAA", type = "Video", flags = 0, created = "", modified = "", bumped = "")

    @Test
    fun linkNamesTheCardOnTheServer() {
        assertEquals("https://mymind.octaviantocan.com/c/00000002VideoAAAAAAAAA", agentLink(card, "https://mymind.octaviantocan.com"))
        assertEquals("https://mymind.octaviantocan.com/c/00000002VideoAAAAAAAAA", agentLink(card, "https://mymind.octaviantocan.com/"))
    }

    @Test
    fun spaceLinkNamesTheSpaceOnTheServer() {
        assertEquals("https://mymind.octaviantocan.com/s/00000002SpaceAAAAAAAAA", spaceAgentLink(Space(id = "00000002SpaceAAAAAAAAA", name = "Kitchen"), "https://mymind.octaviantocan.com/"))
    }
}
