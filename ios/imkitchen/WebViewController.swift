import SafariServices
import UIKit
import WebKit

/// The whole app: a full-screen WKWebView on https://imkitchen.app.
///
/// CAUTION: the "imkitchen-ios" user-agent token is load-bearing twice:
/// - the server hides all upgrade/billing UI, ads and analytics when it sees
///   it (App Store Guideline 3.1.1) — web/shared/src/template.rs;
/// - session validation exact-matches the full UA string, so the token must
///   NEVER change between app releases or every signed-in user is logged out
///   on update — web/shared/src/auth.rs.
final class WebViewController: UIViewController {
    private static let startURL = URL(string: "https://imkitchen.app/")!
    private static let userAgentToken = "imkitchen-ios"

    private var webView: WKWebView!
    private var offlineView: UIView!

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .white

        let configuration = WKWebViewConfiguration()
        // Persistent store: the auth_token cookie survives app restarts.
        configuration.websiteDataStore = .default()
        configuration.allowsInlineMediaPlayback = true
        configuration.userContentController.add(
            WeakScriptMessageHandler(self), name: "keepAwake")

        webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.allowsBackForwardNavigationGestures = true
        webView.scrollView.contentInsetAdjustmentBehavior = .automatic
        if let defaultAgent = webView.value(forKey: "userAgent") as? String {
            webView.customUserAgent = "\(defaultAgent) \(Self.userAgentToken)"
        } else {
            webView.customUserAgent =
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) "
                + "AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 "
                + Self.userAgentToken
        }

        webView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(webView)
        NSLayoutConstraint.activate([
            webView.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            webView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            webView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            webView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        offlineView = makeOfflineView()
        offlineView.isHidden = true
        view.addSubview(offlineView)
        NSLayoutConstraint.activate([
            offlineView.topAnchor.constraint(equalTo: view.topAnchor),
            offlineView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            offlineView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            offlineView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        webView.load(URLRequest(url: Self.startURL))
    }

    override var preferredStatusBarStyle: UIStatusBarStyle { .darkContent }

    private func isAppHost(_ url: URL) -> Bool {
        guard let host = url.host else { return false }
        return host == "imkitchen.app" || host.hasSuffix(".imkitchen.app")
    }

    // MARK: - Offline fallback (cold start; the service worker covers
    // in-session offline once the site has loaded at least once)

    private func makeOfflineView() -> UIView {
        let container = UIView()
        container.translatesAutoresizingMaskIntoConstraints = false
        container.backgroundColor = .white

        let title = UILabel()
        title.text = NSLocalizedString("offline.title", comment: "Offline screen title")
        title.font = .preferredFont(forTextStyle: .title2)
        title.textAlignment = .center

        let message = UILabel()
        message.text = NSLocalizedString("offline.message", comment: "Offline screen message")
        message.font = .preferredFont(forTextStyle: .body)
        message.textColor = .secondaryLabel
        message.textAlignment = .center
        message.numberOfLines = 0

        let retry = UIButton(type: .system)
        retry.setTitle(NSLocalizedString("offline.retry", comment: "Offline retry button"), for: .normal)
        retry.titleLabel?.font = .preferredFont(forTextStyle: .headline)
        retry.addTarget(self, action: #selector(retryLoad), for: .touchUpInside)

        let stack = UIStackView(arrangedSubviews: [title, message, retry])
        stack.axis = .vertical
        stack.spacing = 16
        stack.translatesAutoresizingMaskIntoConstraints = false
        container.addSubview(stack)
        NSLayoutConstraint.activate([
            stack.centerYAnchor.constraint(equalTo: container.centerYAnchor),
            stack.leadingAnchor.constraint(equalTo: container.leadingAnchor, constant: 32),
            stack.trailingAnchor.constraint(equalTo: container.trailingAnchor, constant: -32),
        ])
        return container
    }

    @objc private func retryLoad() {
        offlineView.isHidden = true
        if webView.url != nil {
            webView.reload()
        } else {
            webView.load(URLRequest(url: Self.startURL))
        }
    }
}

// MARK: - WKNavigationDelegate

extension WebViewController: WKNavigationDelegate {
    func webView(
        _ webView: WKWebView,
        decidePolicyFor navigationAction: WKNavigationAction,
        decisionHandler: @escaping (WKNavigationActionPolicy) -> Void
    ) {
        guard let url = navigationAction.request.url else {
            decisionHandler(.cancel)
            return
        }

        if let scheme = url.scheme, ["mailto", "tel", "sms", "facetime"].contains(scheme) {
            UIApplication.shared.open(url)
            decisionHandler(.cancel)
            return
        }

        let isWeb = url.scheme == "http" || url.scheme == "https"
        if isWeb, !isAppHost(url), navigationAction.navigationType == .linkActivated {
            present(SFSafariViewController(url: url), animated: true)
            decisionHandler(.cancel)
            return
        }

        decisionHandler(.allow)
    }

    func webView(
        _ webView: WKWebView,
        didFailProvisionalNavigation navigation: WKNavigation!,
        withError error: Error
    ) {
        let code = (error as NSError).code
        if code == NSURLErrorNotConnectedToInternet
            || code == NSURLErrorNetworkConnectionLost
            || code == NSURLErrorTimedOut
        {
            offlineView.isHidden = false
        }
    }
}

// MARK: - WKUIDelegate (target=_blank)

extension WebViewController: WKUIDelegate {
    func webView(
        _ webView: WKWebView,
        createWebViewWith configuration: WKWebViewConfiguration,
        for navigationAction: WKNavigationAction,
        windowFeatures: WKWindowFeatures
    ) -> WKWebView? {
        if let url = navigationAction.request.url {
            if isAppHost(url) {
                webView.load(navigationAction.request)
            } else {
                present(SFSafariViewController(url: url), animated: true)
            }
        }
        return nil
    }
}

// MARK: - keepAwake bridge (cooking mode screen wake lock,
// see templates/_cooking.html)

extension WebViewController: WKScriptMessageHandler {
    func userContentController(
        _ userContentController: WKUserContentController,
        didReceive message: WKScriptMessage
    ) {
        guard message.name == "keepAwake" else { return }
        UIApplication.shared.isIdleTimerDisabled = (message.body as? Bool) ?? false
    }
}

/// WKUserContentController retains its handlers; this proxy avoids the
/// resulting retain cycle on the view controller.
private final class WeakScriptMessageHandler: NSObject, WKScriptMessageHandler {
    private weak var delegate: WKScriptMessageHandler?

    init(_ delegate: WKScriptMessageHandler) {
        self.delegate = delegate
    }

    func userContentController(
        _ userContentController: WKUserContentController,
        didReceive message: WKScriptMessage
    ) {
        delegate?.userContentController(userContentController, didReceive: message)
    }
}
