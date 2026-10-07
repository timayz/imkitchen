package app.imkitchen.db.put

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `Db.put`; see `DbStore` for the semantics. */
abstract class AbsDbPutMethodIDL :
    AbsSparklingIDLMethod<AbsDbPutMethodIDL.Params, AbsDbPutMethodIDL.Result>() {

    @IDLMethodName(name = "Db.put", params = ["collection", "key", "value"], results = ["ok"])
    final override val name: String = "Db.put"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "collection")
        val collection: String

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "key")
        val key: String

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "value")
        val value: String
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "ok")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "ok")
        var ok: Boolean?
    }
}
