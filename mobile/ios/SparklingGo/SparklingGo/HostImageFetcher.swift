import Foundation
import Lynx
import SDWebImage
import Sparkling
import SparklingMethod

/// Loads what an `<image>` of a page points at.
///
/// Sparkling 2.0.1 only knows the files of the app bundle on iOS and never
/// answers for anything else, so an image inlined in a Lynx bundle (`data:`,
/// what rspeedy does with the small ones, the tab icons for instance) or one
/// behind a URL stays blank. Those two are loaded here, the bundle files are
/// left to Sparkling.
final class HostImageFetcher: NSObject, LynxImageFetcher {
    static let shared = HostImageFetcher()

    func loadImage(with url: URL, size targetSize: CGSize, contextInfo: [AnyHashable : Any]?, completion completionBlock: @escaping LynxImageLoadCompletionBlock) -> () -> Void {
        if url.scheme == "data" {
            completionBlock(HostImageFetcher.image(fromDataURL: url.absoluteString), nil, url)
            return {}
        }
        if url.scheme == "http" || url.scheme == "https" {
            let operation = SDWebImageManager.shared.loadImage(with: url, options: [], progress: nil) { image, _, error, _, _, _ in
                completionBlock(image, error, url)
            }
            return {
                operation?.cancel()
            }
        }
        let loader = SPKKit.DIContainer.resolve(SPKResourceLoaderProtocol.self)
        let task = loader?.loadImage(withURL: url, completion: { image, error in
            completionBlock(image, error, url)
        })
        return {
            task?.cancel()
        }
    }

    /// `data:image/png;base64,…`
    static func image(fromDataURL string: String) -> UIImage? {
        guard let comma = string.firstIndex(of: ","),
              string[..<comma].hasSuffix(";base64"),
              let data = Data(base64Encoded: String(string[string.index(after: comma)...]), options: .ignoreUnknownCharacters) else {
            return nil
        }
        return UIImage(data: data)
    }
}
