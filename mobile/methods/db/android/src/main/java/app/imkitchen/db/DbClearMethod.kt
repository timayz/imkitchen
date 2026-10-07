package app.imkitchen.db

import app.imkitchen.db.clear.AbsDbClearMethodIDL
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

class DbClearMethod : AbsDbClearMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val collection = params.collection
        DbStore.execute {
            try {
                DbStore.clear(context, collection)
                callback.onSuccess(Result::class.java.createXModel().apply { this.ok = true })
            } catch (e: Exception) {
                callback.onFailure(IDLBridgeMethod.FAIL, "Db.clear failed: $e")
            }
        }
    }
}
