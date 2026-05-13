using System;
using System.Diagnostics;
using System.IO;
using Photino.NET;
using System.Drawing;

namespace TList;

class Program
{
    [STAThread]
    static void Main(string[] args)
    {
        // WebView2のEdgeミニメニュー（テキスト選択時の翻訳ボタン等）を抑制
        Environment.SetEnvironmentVariable(
            "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
            "--disable-features=msEdgeMiniMenu,msEdgeAskMeAnything,TextSuggestionsForMiniMenu");

        var htmlPath = Path.Combine(AppContext.BaseDirectory, "TList.html");

        var iconPath = Path.Combine(AppContext.BaseDirectory, "icon.ico");
        var window = new PhotinoWindow()
            .SetTitle("TList")
            .SetIconFile(iconPath)
            .SetUseOsDefaultSize(false)
            .SetSize(new Size(1000, 860))
            .SetMinSize(320, 480)
            .Center()
            .Load(htmlPath);

        // JS -> C# メッセージハンドラ
        window.RegisterWebMessageReceivedHandler((sender, message) =>
        {
            if (message.StartsWith("open-folder:"))
            {
                var path = message["open-folder:".Length..];
                try
                {
                    var psi = new ProcessStartInfo
                    {
                        FileName = path,
                        UseShellExecute = true
                    };
                    Process.Start(psi);
                    window.SendWebMessage("folder-result:success");
                }
                catch (Exception ex)
                {
                    window.SendWebMessage($"folder-result:error:{ex.Message}");
                }
            }
            else if (message.StartsWith("open-url:"))
            {
                var url = message["open-url:".Length..];
                try
                {
                    var psi = new ProcessStartInfo
                    {
                        FileName = url,
                        UseShellExecute = true
                    };
                    Process.Start(psi);
                    window.SendWebMessage("url-result:success");
                }
                catch (Exception ex)
                {
                    window.SendWebMessage($"url-result:error:{ex.Message}");
                }
            }
        });

        window.WaitForClose();
    }
}
