package app.imkitchen.keepawake

import android.view.WindowManager
import app.imkitchen.keepawake.setenabled.AbsSetEnabledMethodIDL
import com.tiktok.sparkling.hybridkit.service.HybridActivityStackManager
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel

/**
 * `KeepAwake.setEnabled({enabled})`: keeps the screen on for the container
 * that called it (the cooking screen). The flag dies with that activity, so
 * leaving the screen always restores the default.
 */
class KeepAwakeSetEnabledMethod : AbsSetEnabledMethodIDL() {

    override fun handle(
        params: IDLMethodSetEnabledInputModel,
        callback: CompletionBlock<IDLMethodSetEnabledResultModel>,
        type: BridgePlatformType
    ) {
        val activity = HybridActivityStackManager.getTopActivity()
            ?: return callback.onFailure(0, "No foreground activity")

        activity.runOnUiThread {
            if (params.enabled) {
                activity.window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            } else {
                activity.window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            }
        }

        callback.onSuccess(
            IDLMethodSetEnabledResultModel::class.java.createXModel().apply { success = true }
        )
    }
}
