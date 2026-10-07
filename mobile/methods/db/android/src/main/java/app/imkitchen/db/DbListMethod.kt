package app.imkitchen.db

import app.imkitchen.db.list.AbsDbListMethodIDL
import com.tiktok.sparkling.method.registry.core.BridgePlatformType
import com.tiktok.sparkling.method.registry.core.IDLBridgeMethod
import com.tiktok.sparkling.method.registry.core.model.idl.CompletionBlock
import com.tiktok.sparkling.method.registry.core.utils.createXModel
import com.tiktok.sparkling.method.runtime.depend.BridgeBaseRuntime

class DbListMethod : AbsDbListMethodIDL() {

    override fun handle(params: Params, callback: CompletionBlock<Result>, type: BridgePlatformType) {
        val context = BridgeBaseRuntime.applicationContext
            ?: return callback.onFailure(IDLBridgeMethod.FAIL, "Context not provided in host")
        val collection = params.collection
        val prefix = params.prefix
        if (collection.isEmpty()) {
            return callback.onFailure(IDLBridgeMethod.INVALID_PARAM, "Db.list: empty collection")
        }
        DbStore.execute {
            try {
                val json = DbStore.list(context, collection, prefix)
                callback.onSuccess(Result::class.java.createXModel().apply { this.json = json })
            } catch (e: Exception) {
                callback.onFailure(IDLBridgeMethod.FAIL, "Db.list failed: $e")
            }
        }
    }
}
