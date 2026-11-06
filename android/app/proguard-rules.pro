# The release app is shrunk and optimised by R8 for speed (Tavi, 4 November 2026). Names are kept, so crash reports
# and logs read as the source does; the libraries that use reflection (kotlinx.serialization, WorkManager, Coil,
# OkHttp) ship their own keep rules.
-dontobfuscate

# The share moment's page calls InvApp.post (WebMoment.Bridge), so the method keeps its name.
-keepclassmembers class com.octaviantocan.mymind.ui.moment.WebMoment$Bridge {
    @android.webkit.JavascriptInterface <methods>;
}
