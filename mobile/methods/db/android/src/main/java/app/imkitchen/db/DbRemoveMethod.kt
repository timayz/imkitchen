package app.imkitchen.db

import app.imkitchen.db.remove.AbsDbRemoveMethodIDL
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

class DbRemoveMethod : AbsDbRemoveMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val collection = params.collection
        val key = params.key
        if (collection.isEmpty() || key.isEmpty()) {
            return callback.onFailure(IDLBridgeMethod.INVALID_PARAM, "Db.remove: empty collection/key")
        }
        DbStore.execute {
            try {
                DbStore.remove(context, collection, key)
                callback.onSuccess(Result::class.java.createXModel().apply { this.ok = true })
            } catch (e: Exception) {
                callback.onFailure(IDLBridgeMethod.FAIL, "Db.remove failed: $e")
            }
        }
    }
}
