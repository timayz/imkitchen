package app.imkitchen.db.get

import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodName
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamField
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodParamModel
import com.tiktok.sparkling.method.registry.core.annotation.IDLMethodResultModel
import com.tiktok.sparkling.method.registry.core.base.AbsSparklingIDLMethod
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseParamModel
import com.tiktok.sparkling.method.registry.core.model.idl.IDLMethodBaseResultModel

/** IDL of `Db.get`; see `DbStore` for the semantics. */
abstract class AbsDbGetMethodIDL :
    AbsSparklingIDLMethod<AbsDbGetMethodIDL.Params, AbsDbGetMethodIDL.Result>() {

    @IDLMethodName(name = "Db.get", params = ["collection", "key"], results = ["value"])
    final override val name: String = "Db.get"

    @IDLMethodParamModel
    interface Params : IDLMethodBaseParamModel {

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "collection")
        val collection: String

        @get:IDLMethodParamField(required = true, isGetter = true, keyPath = "key")
        val key: String
    }

    @IDLMethodResultModel
    interface Result : IDLMethodBaseResultModel {

        @get:IDLMethodParamField(required = false, isGetter = true, keyPath = "value")
        @set:IDLMethodParamField(required = false, isGetter = false, keyPath = "value")
        var value: String?
    }
}
