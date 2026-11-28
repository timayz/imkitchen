// Copyright 2025 The Sparkling Authors. All rights reserved.
// Licensed under the Apache License Version 2.0 that can be found in the
// LICENSE file in the root directory of this source tree.

import Foundation
import SwiftUI
import Lynx
import Sparkling

class SparklingLynxElement: SPKLynxElement {
    var lynxElementName: String
    
    var lynxElementClassName: AnyClass
    
    init(lynxElementName: String, lynxElementClassName: AnyClass) {
        self.lynxElementName = lynxElementName
        self.lynxElementClassName = lynxElementClassName
    }
}

/// What every container of the app is opened with, the first one (below) and
/// the ones a page pushes (`RouterServiceImpl`).
enum HostPage {
    static func context() -> SPKContext {
        let context = SPKContext()
        // The bundled Lynx has no `<input>`: the host provides it.
        context.customUIElements = [
            SparklingLynxElement(lynxElementName: "input", lynxElementClassName: LynxInput.self)
        ]
        context.containerLifecycleDelegate = HostGlobalProps.shared
        return context
    }

    /// The page the app opens on. The colours are `--color-paper` in
    /// src/styles/tokens.css: what shows behind the status bar and until the
    /// page has rendered. Pushed pages get theirs from src/lib/nav.ts.
    static var mainURL: String {
        let dark = UITraitCollection.current.userInterfaceStyle == .dark
        let paper = dark ? "%231b140c" : "%23fbf5e9"
        return "hybrid://lynxview_page?bundle=main.lynx.bundle&hide_nav_bar=1&hide_loading=1"
            + "&container_bg_color=\(paper)&loading_bg_color=\(paper)"
            + "&status_font_mode=\(dark ? "light" : "dark")"
    }
}

/// Hands `lynx.__globalProps` to every page before it loads.
///
/// Sparkling 2.0.1 builds the global props on iOS but never passes them to
/// the Lynx view (fixed upstream in 2.1), so a page finds
/// `lynx.__globalProps` null. Only what the pages read is provided: the
/// fields of `GlobalProps` in src/typing.d.ts.
final class HostGlobalProps: NSObject, SPKContainerLifecycleProtocol {
    static let shared = HostGlobalProps()

    func containerWillStartLoading(_ contanier: SPKContainerProtocol) {
        guard let view = contanier.kitView as? LynxView else {
            return
        }
        let props = HostGlobalProps.props(forURL: contanier.originURL)
        view.updateGlobalProps(with: props)
    }

    static func props(forURL url: URL?) -> [String: Any] {
        var queryItems: [String: String] = [:]
        if let url = url, let components = URLComponents(url: url, resolvingAgainstBaseURL: false) {
            components.queryItems?.forEach { item in
                queryItems[item.name] = item.value ?? ""
            }
        }
        let window = UIApplication.shared.windows.first
        return [
            "os": "ios",
            "osVersion": UIDevice.current.systemVersion,
            "language": Locale.preferredLanguages.first ?? "",
            "theme": UITraitCollection.current.userInterfaceStyle == .dark ? "dark" : "light",
            "bottomHeight": window?.safeAreaInsets.bottom ?? 0,
            "queryItems": queryItems,
        ]
    }
}

struct SPKSwiftVC: UIViewControllerRepresentable {
    @State private var state_frame: CGRect
    
    init(state_frame: CGRect = .zero) {
        self.state_frame = state_frame
    }
    
    func makeUIViewController(context: Context) -> some UIViewController {
        let vc = SPKRouter.create(withURL: HostPage.mainURL, context: HostPage.context(), frame: self.state_frame)
        let naviVC = UINavigationController(rootViewController: vc)
        return naviVC
    }
    
    func updateUIViewController(_ uiViewController: UIViewControllerType, context: Context) {
        
    }
}


struct DemoVC: View {
    var body: some View {
        GeometryReader { geometry in
            SPKSwiftVC(state_frame: geometry.frame(in: .local))
        }
        
    }
}
